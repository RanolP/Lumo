use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    diagnostics::Diagnostic,
    hir,
    lexer::Span,
    lir, lst,
    parser::{self, ParseOutput},
    typecheck,
    types::{CapEntry, CapRef, ExprId, Pattern, TypeExpr},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub lossless: lst::lossless::ParseOutput,
    pub file: crate::lst::File,
    pub errors: Vec<parser::ParseError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryStats {
    pub parse_computes: usize,
    pub lower_computes: usize,
    pub diagnostics_computes: usize,
}

impl QueryStats {
    fn new() -> Self {
        Self {
            parse_computes: 0,
            lower_computes: 0,
            diagnostics_computes: 0,
        }
    }
}

impl Default for QueryStats {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct FileEntry {
    source: String,
    source_hash: u64,
    parsed_at_hash: Option<u64>,
    parse: Option<ParseResult>,
    lowered_hir_at_hash: Option<u64>,
    lowered_hir: Option<hir::File>,
    lowered_at_hash: Option<u64>,
    lowered: Option<lir::File>,
    diagnostics_at_hash: Option<u64>,
    diagnostics: Option<Vec<Diagnostic>>,
}

impl FileEntry {
    fn new(source: String) -> Self {
        let source_hash = hash_text(&source);
        Self {
            source,
            source_hash,
            parsed_at_hash: None,
            parse: None,
            lowered_hir_at_hash: None,
            lowered_hir: None,
            lowered_at_hash: None,
            lowered: None,
            diagnostics_at_hash: None,
            diagnostics: None,
        }
    }

    fn set_source(&mut self, source: String) {
        let new_hash = hash_text(&source);
        if new_hash == self.source_hash {
            self.source = source;
            return;
        }

        self.source = source;
        self.source_hash = new_hash;
        self.parsed_at_hash = None;
        self.parse = None;
        self.lowered_hir_at_hash = None;
        self.lowered_hir = None;
        self.lowered_at_hash = None;
        self.lowered = None;
        self.diagnostics_at_hash = None;
        self.diagnostics = None;
    }
}

#[derive(Debug)]
pub struct QueryEngine {
    files: HashMap<String, FileEntry>,
    stats: QueryStats,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            stats: QueryStats::new(),
        }
    }

    pub fn set_file(&mut self, file: impl Into<String>, source: impl Into<String>) {
        let file = file.into();
        let source = source.into();
        match self.files.get_mut(&file) {
            Some(entry) => entry.set_source(source),
            None => {
                self.files.insert(file, FileEntry::new(source));
            }
        }
    }

    pub fn remove_file(&mut self, file: &str) -> bool {
        self.files.remove(file).is_some()
    }

    pub fn parse(&mut self, file: &str) -> Option<ParseResult> {
        let entry = self.files.get_mut(file)?;

        if entry.parsed_at_hash == Some(entry.source_hash) {
            return entry.parse.clone();
        }

        let lossless = lst::lossless::parse(&entry.source);
        let parsed: ParseOutput = parser::parse_lossless(&lossless);

        let parse = ParseResult {
            lossless,
            file: parsed.file,
            errors: parsed.errors,
        };

        entry.parsed_at_hash = Some(entry.source_hash);
        entry.parse = Some(parse.clone());

        self.stats.parse_computes += 1;

        Some(parse)
    }

    pub fn lower_hir(&mut self, file: &str) -> Option<hir::File> {
        let source_hash = self.files.get(file)?.source_hash;
        if self.files.get(file)?.lowered_hir_at_hash == Some(source_hash) {
            return self.files.get(file)?.lowered_hir.clone();
        }

        let parsed = self.parse(file)?;
        let lowered = hir::lower(&parsed.file);

        let entry = self.files.get_mut(file)?;
        entry.lowered_hir_at_hash = Some(entry.source_hash);
        entry.lowered_hir = Some(lowered.clone());

        Some(lowered)
    }

    pub fn lower(&mut self, file: &str) -> Option<lir::File> {
        let source_hash = self.files.get(file)?.source_hash;
        if self.files.get(file)?.lowered_at_hash == Some(source_hash) {
            return self.files.get(file)?.lowered.clone();
        }

        let lowered_hir = self.lower_hir(file)?;
        let lowered = lir::lower(&lowered_hir);

        let entry = self.files.get_mut(file)?;
        entry.lowered_at_hash = Some(entry.source_hash);
        entry.lowered = Some(lowered.clone());
        self.stats.lower_computes += 1;

        Some(lowered)
    }

    /// Lower a group of files as a single module.
    /// Parses each file individually (reusing per-file cache), merges HIR,
    /// then lowers the merged HIR to a single LIR with inferred caps applied.
    ///
    /// The pipeline:
    /// 1. Merge HIR → lower to LIR
    /// 2. Typecheck to get per-Perform type_args resolutions
    /// 3. Patch Perform nodes with resolved type_args (e.g. Add[] → Add[Number])
    /// 4. Resolve default cap impls (Perform → Ident for caps with matching impls)
    /// 5. Re-run cap inference on patched LIR
    pub fn lower_module(&mut self, files: &[&str]) -> Option<lir::File> {
        let mut hir_files = Vec::new();
        for file in files {
            hir_files.push(self.lower_hir(file)?);
        }
        let merged = hir::merge_files(&hir_files);
        let mut lowered = lir::lower(&merged);

        // Phase 0: Rewrite value method calls (e.g. "asdf".len() → String.len("asdf"))
        rewrite_value_method_calls(&mut lowered);

        // Phase 1: Typecheck to get perform_for_types
        let (_, perform_for_types) = typecheck::infer_caps_for_file(&lowered);
        // Phase 2: Patch Perform nodes with resolved type_args
        patch_perform_type_args(&mut lowered, &perform_for_types);
        // Phase 2.5: Fill default type_args for Perform/Handle with empty type_args
        fill_default_type_args(&mut lowered);
        // Phase 3: Re-run cap inference on patched LIR
        let (inferred, _) = typecheck::infer_caps_for_file(&lowered);
        typecheck::apply_inferred_caps(&mut lowered, &inferred);

        // Phase 4: LTO — monomorphize cap-resolved fns
        let lto_errors = crate::lto::optimize(&mut lowered);
        if !lto_errors.is_empty() {
            // Hard errors (e.g. #[inline(always)] on unresolvable fn) — abort.
            return None;
        }
        // Phase 4': Re-typecheck (clones changed cap requirements)
        let (inferred, _) = typecheck::infer_caps_for_file(&lowered);
        typecheck::apply_inferred_caps(&mut lowered, &inferred);

        Some(lowered)
    }

    /// Compile entry files with transitive `use` resolution.
    ///
    /// The `resolve` callback maps a use-path (e.g. `["libstd", "io"]`) to
    /// a `(filename, source)` pair. Resolution is applied iteratively until
    /// all dependencies are loaded, then all files are merged via `lower_module`.
    pub fn compile_with_deps<F>(
        &mut self,
        entry_files: &[&str],
        mut resolve: F,
    ) -> Option<lir::File>
    where
        F: FnMut(&[String]) -> Option<(String, String)>,
    {
        // Preserve deterministic insertion order: the Vec records the order
        // files were discovered (entry files first, then deps in BFS order),
        // the HashSet is used only for O(1) duplicate-check.
        let mut ordered_files: Vec<String> = entry_files.iter().map(|f| f.to_string()).collect();
        let mut seen: HashSet<String> = ordered_files.iter().cloned().collect();
        let mut pending: VecDeque<String> = ordered_files.iter().cloned().collect();

        while let Some(file) = pending.pop_front() {
            let parsed = self.parse(&file)?;
            for use_path in collect_use_paths(&parsed.file) {
                if let Some((filename, source)) = resolve(&use_path) {
                    if seen.insert(filename.clone()) {
                        self.set_file(&filename, source.clone());
                        ordered_files.push(filename.clone());
                        pending.push_back(filename);
                    }
                }
            }
        }

        let file_refs: Vec<&str> = ordered_files.iter().map(|s| s.as_str()).collect();
        self.lower_module(&file_refs)
    }

    /// Run HIR-level checks (name resolution, arity, duplicates, patterns).
    pub fn check_hir(&mut self, file: &str) -> Option<Vec<hir::check::CheckError>> {
        let hir_file = self.lower_hir(file)?;
        Some(hir::check::check_file(&hir_file))
    }

    pub fn diagnostics(&mut self, file: &str) -> Option<Vec<Diagnostic>> {
        let source_hash = self.files.get(file)?.source_hash;
        if self.files.get(file)?.diagnostics_at_hash == Some(source_hash) {
            return self.files.get(file)?.diagnostics.clone();
        }

        let parsed = self.parse(file)?;
        let mut diags = parsed
            .errors
            .iter()
            .map(|e| Diagnostic {
                start: e.span.start,
                end: e.span.end,
                message: e.message.clone(),
            })
            .collect::<Vec<_>>();
        // HIR errors (e.g. invalid patterns)
        if let Some(hir_file) = self.lower_hir(file) {
            diags.extend(hir_file.errors.iter().map(|e| Diagnostic {
                start: e.span.start,
                end: e.span.end,
                message: e.message.clone(),
            }));

            // HIR check errors (name resolution, arity, etc.)
            let check_errors = hir::check::check_file(&hir_file);
            diags.extend(check_errors.into_iter().map(|e| Diagnostic {
                start: e.span.start,
                end: e.span.end,
                message: e.message,
            }));
        }

        let lowered = self.lower(file)?;

        // LIR structural validation (dev-mode warnings)
        let lir_warnings = lir::validate::validate(&lowered);
        if !lir_warnings.is_empty() {
            for w in &lir_warnings {
                let span = w
                    .expr_id
                    .and_then(|id| lowered.spans.get(id.0 as usize).copied())
                    .unwrap_or(Span::new(0, 0));
                diags.push(Diagnostic {
                    start: span.start,
                    end: span.end,
                    message: format!("[LIR] {}", w.message),
                });
            }
        }

        let span_map = build_lir_span_map(&lowered);
        let type_errors = typecheck::typecheck_file(&lowered);
        diags.extend(type_errors.into_iter().map(|e| {
            let span = e
                .span
                .or_else(|| span_map.get(&e.node_id).copied())
                .unwrap_or(Span::new(0, 0));
            Diagnostic {
                start: span.start,
                end: span.end,
                message: e.message,
            }
        }));

        let entry = self.files.get_mut(file)?;
        entry.diagnostics_at_hash = Some(entry.source_hash);
        entry.diagnostics = Some(diags.clone());
        self.stats.diagnostics_computes += 1;

        Some(diags)
    }

    pub fn stats(&self) -> QueryStats {
        self.stats.clone()
    }
}

fn build_lir_span_map(file: &lir::File) -> HashMap<u64, Span> {
    let mut out = HashMap::new();
    // Populate from the spans side-table: ExprId(i) → file.spans[i]
    for (i, span) in file.spans.iter().enumerate() {
        out.insert(i as u64, *span);
    }
    out
}

fn hash_text(text: &str) -> u64 {
    let mut state = 0xcbf29ce484222325_u64;
    for b in text.as_bytes() {
        state ^= *b as u64;
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

fn collect_use_paths(file: &crate::lst::File) -> Vec<Vec<String>> {
    file.items
        .iter()
        .filter_map(|item| {
            if let lst::Item::Use(u) = item {
                Some(u.path.clone())
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Value method call rewriting
// ---------------------------------------------------------------------------

/// Compute the impl const name for an impl declaration (mirrors ts.rs::impl_const_name).
fn impl_const_name_for(impl_decl: &lir::ImplDecl) -> String {
    if let Some(name) = &impl_decl.name {
        name.clone()
    } else if let Some(cap) = &impl_decl.capability {
        let target = impl_decl.target_type.value.display();
        let cap = cap.value.display();
        format!("__impl_{target}_{cap}")
    } else {
        impl_decl.target_type.value.display()
    }
}

/// Context for the value method call rewrite pass.
struct RewriteCtx {
    /// (target_type, method_name) → impl_const_name — unambiguous method dispatch table
    resolved: HashMap<(String, String), String>,
    /// fn_name → return type name (simple Named types only)
    fn_return_types: HashMap<String, String>,
    /// (obj_name_or_impl_const, method_name) → return type name
    method_return_types: HashMap<(String, String), String>,
    /// variant_name → [field type names] for data constructor pattern matching
    data_variant_fields: HashMap<String, Vec<String>>,
}

/// Extract a simple type name from a TypeExpr, if it's a plain Named type.
fn simple_type_name(ty: &TypeExpr) -> Option<String> {
    match ty {
        TypeExpr::Named(n) => Some(n.clone()),
        _ => None,
    }
}

/// Rewrite value method calls: `expr.method(args)` → `ImplConst.method(expr, args)`.
///
/// Strip type arguments: `"List[T]"` → `"List"`, `"Number"` → `"Number"`.
fn base_type_name(s: &str) -> &str {
    if let Some(idx) = s.find('[') { &s[..idx] } else { s }
}

/// Builds a method table from all `Impl` items, then walks function and impl method bodies.
/// For each `Member { object, field }` where the object's type has a matching impl method,
/// replaces the Member with `Apply(Member(Ident(impl_const), field), original_object)`.
fn rewrite_value_method_calls(file: &mut lir::File) {
    // Build method table: (target_type, method_name) → [impl_const_name]
    let mut method_table: HashMap<(String, String), Vec<String>> = HashMap::new();
    let mut method_return_types: HashMap<(String, String), String> = HashMap::new();

    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            let target_display = impl_decl.target_type.value.display();
            let target = base_type_name(&target_display).to_owned();
            let const_name = impl_const_name_for(impl_decl);
            for method in &impl_decl.methods {
                method_table
                    .entry((target.clone(), method.name.clone()))
                    .or_default()
                    .push(const_name.clone());
                if let Some(ret_ty) = &method.return_type {
                    if let Some(ty) = simple_type_name(&ret_ty.value) {
                        method_return_types
                            .insert((const_name.clone(), method.name.clone()), ty);
                    }
                }
            }
        }
    }

    if method_table.is_empty() {
        return;
    }

    // Filter to unambiguous entries only
    let resolved: HashMap<(String, String), String> = method_table
        .into_iter()
        .filter_map(|(key, impls)| {
            if impls.len() == 1 {
                Some((key, impls.into_iter().next().unwrap()))
            } else {
                None
            }
        })
        .collect();

    if resolved.is_empty() {
        return;
    }

    // Build return type tables for functions and cap operations
    let mut fn_return_types: HashMap<String, String> = HashMap::new();
    let mut data_variant_fields: HashMap<String, Vec<String>> = HashMap::new();

    for item in &file.items {
        match item {
            lir::Item::Fn(f) => {
                if let Some(ret_ty) = &f.return_type {
                    if let Some(ty) = simple_type_name(&ret_ty.value) {
                        fn_return_types.insert(f.name.clone(), ty);
                    }
                }
            }
            lir::Item::Data(d) => {
                for variant in &d.variants {
                    let field_types: Vec<Option<String>> =
                        variant.payload.iter().map(|p| simple_type_name(&p.value)).collect();
                    if field_types.iter().all(|t| t.is_some()) {
                        data_variant_fields.insert(
                            variant.name.clone(),
                            field_types.into_iter().map(|t| t.unwrap()).collect(),
                        );
                    }
                }
            }
            lir::Item::Cap(c) => {
                for op in &c.operations {
                    if let Some(ret_ty) = &op.return_type {
                        if let Some(ty) = simple_type_name(&ret_ty.value) {
                            method_return_types
                                .insert((c.name.clone(), op.name.clone()), ty);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let ctx = RewriteCtx {
        resolved,
        fn_return_types,
        method_return_types,
        data_variant_fields,
    };

    // Split borrow: take spans out so we can mutate items and spans independently
    let mut spans = std::mem::take(&mut file.spans);

    for item in &mut file.items {
        match item {
            lir::Item::Fn(f) => {
                let scope = build_param_scope(&f.params);
                rewrite_method_calls_in_expr(&mut f.value, &ctx, &scope, &mut spans);
            }
            lir::Item::Impl(impl_decl) => {
                let method_scopes: Vec<HashMap<String, String>> = impl_decl
                    .methods
                    .iter()
                    .map(|m| build_param_scope(&m.params))
                    .collect();
                for (m, scope) in impl_decl.methods.iter_mut().zip(method_scopes.iter()) {
                    rewrite_method_calls_in_expr(&mut m.value, &ctx, scope, &mut spans);
                }
            }
            _ => {}
        }
    }

    file.spans = spans;
}

fn build_param_scope(params: &[lir::Param]) -> HashMap<String, String> {
    params
        .iter()
        .map(|p| (p.name.clone(), p.ty.value.display()))
        .collect()
}

/// Try to determine the type name of an expression from syntactic information.
fn determine_expr_type(
    expr: &lir::Expr,
    scope: &HashMap<String, String>,
    ctx: &RewriteCtx,
) -> Option<String> {
    match expr {
        lir::Expr::String { .. } => Some("String".to_string()),
        lir::Expr::Number { .. } => Some("Number".to_string()),
        lir::Expr::Ident { name, .. } => scope.get(name).cloned(),
        lir::Expr::Produce { expr, .. } | lir::Expr::Ann { expr, .. } => {
            determine_expr_type(expr, scope, ctx)
        }
        lir::Expr::Apply { .. } => determine_apply_return_type(expr, ctx),
        _ => None,
    }
}

/// Determine the return type of a (possibly curried) Apply chain by finding the
/// root callee (Ident for fn calls, Member for method calls) and looking up its
/// declared return type.
fn determine_apply_return_type(expr: &lir::Expr, ctx: &RewriteCtx) -> Option<String> {
    // Peel off Apply layers to find the root callee
    let mut current = expr;
    while let lir::Expr::Apply { callee, .. } = current {
        current = callee;
    }
    // Also peel through Force/Produce/Thunk wrappers
    loop {
        match current {
            lir::Expr::Force { expr, .. }
            | lir::Expr::Produce { expr, .. }
            | lir::Expr::Thunk { expr, .. } => current = expr,
            _ => break,
        }
    }

    match current {
        lir::Expr::Ident { name, .. } => ctx.fn_return_types.get(name).cloned(),
        lir::Expr::Member { object, field, .. } => {
            if let lir::Expr::Ident { name, .. } = object.as_ref() {
                ctx.method_return_types
                    .get(&(name.clone(), field.clone()))
                    .cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Add type bindings from a match pattern to the scope, using data variant field types.
fn add_pattern_bindings(
    pattern: &Pattern,
    variant_fields: &HashMap<String, Vec<String>>,
    scope: &mut HashMap<String, String>,
) {
    match pattern {
        Pattern::Ctor { name, args } => {
            if let Some(field_types) = variant_fields.get(name.as_str()) {
                for (arg, ty) in args.iter().zip(field_types.iter()) {
                    match arg {
                        Pattern::Bind(binding_name) => {
                            scope.insert(binding_name.clone(), ty.clone());
                        }
                        Pattern::Ctor { .. } => {
                            add_pattern_bindings(arg, variant_fields, scope);
                        }
                        Pattern::Wildcard => {}
                    }
                }
            }
        }
        Pattern::Bind(_) | Pattern::Wildcard => {}
    }
}

fn alloc_expr_id(spans: &mut Vec<crate::lexer::Span>, span: crate::lexer::Span) -> ExprId {
    let id = ExprId(spans.len() as u32);
    spans.push(span);
    id
}

/// Recursively walk and rewrite method calls in an expression.
fn rewrite_method_calls_in_expr(
    expr: &mut lir::Expr,
    ctx: &RewriteCtx,
    scope: &HashMap<String, String>,
    spans: &mut Vec<crate::lexer::Span>,
) {
    match expr {
        lir::Expr::Member { id, object, field } => {
            // First recurse into the object
            rewrite_method_calls_in_expr(object, ctx, scope, spans);

            // Then check if this member access should be rewritten
            if let Some(type_name) = determine_expr_type(object, scope, ctx) {
                let key = (base_type_name(&type_name).to_owned(), field.clone());
                if let Some(impl_const) = ctx.resolved.get(&key) {
                    let member_id = *id;
                    let span = spans.get(member_id.0 as usize).copied().unwrap_or(crate::lexer::Span::new(0, 0));

                    // Take original object, replace with Ident(impl_const)
                    let original_object = std::mem::replace(
                        object.as_mut(),
                        lir::Expr::Ident {
                            id: alloc_expr_id(spans, span),
                            name: impl_const.clone(),
                        },
                    );

                    // Wrap: Member(Ident(impl_const), field) → Apply(Member(...), original_object)
                    let member_node = std::mem::replace(
                        expr,
                        lir::Expr::Error { id: ExprId(0) },
                    );
                    *expr = lir::Expr::Apply {
                        id: alloc_expr_id(spans, span),
                        callee: Box::new(member_node),
                        arg: Box::new(original_object),
                    };
                    return;
                }
            }
        }
        lir::Expr::Apply { callee, arg, .. } => {
            rewrite_method_calls_in_expr(callee, ctx, scope, spans);
            rewrite_method_calls_in_expr(arg, ctx, scope, spans);
        }
        lir::Expr::Let { name, value, body, .. } => {
            rewrite_method_calls_in_expr(value, ctx, scope, spans);
            // Extend scope with Let binding type (infer from value)
            let mut inner_scope = scope.clone();
            if let Some(ty) = determine_expr_type(value, scope, ctx) {
                inner_scope.insert(name.clone(), ty);
            }
            rewrite_method_calls_in_expr(body, ctx, &inner_scope, spans);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            rewrite_method_calls_in_expr(scrutinee, ctx, scope, spans);
            for arm in arms {
                let mut arm_scope = scope.clone();
                add_pattern_bindings(&arm.pattern, &ctx.data_variant_fields, &mut arm_scope);
                rewrite_method_calls_in_expr(&mut arm.body, ctx, &arm_scope, spans);
            }
        }
        lir::Expr::Lambda { body, .. } => {
            rewrite_method_calls_in_expr(body, ctx, scope, spans);
        }
        lir::Expr::Handle { handler, body, .. } => {
            rewrite_method_calls_in_expr(handler, ctx, scope, spans);
            rewrite_method_calls_in_expr(body, ctx, scope, spans);
        }
        lir::Expr::Thunk { expr: inner, .. }
        | lir::Expr::Produce { expr: inner, .. }
        | lir::Expr::Force { expr: inner, .. }
        | lir::Expr::Unroll { expr: inner, .. }
        | lir::Expr::Roll { expr: inner, .. }
        | lir::Expr::Ann { expr: inner, .. } => {
            rewrite_method_calls_in_expr(inner, ctx, scope, spans);
        }
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                rewrite_method_calls_in_expr(&mut e.body, ctx, scope, spans);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                rewrite_method_calls_in_expr(a, ctx, scope, spans);
            }
        }
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Perform { .. }
        | lir::Expr::Error { .. } => {}
    }
}

// ---------------------------------------------------------------------------
// Default cap impl resolution
// ---------------------------------------------------------------------------

/// Patch `Perform` nodes with resolved type_args based on typechecker output.
/// e.g. `Perform { cap: "Add", type_args: [] }` → `Perform { cap: "Add", type_args: ["Number"] }`
/// when the typechecker resolved `Self = Number` at that site.
fn patch_perform_type_args(file: &mut lir::File, perform_for_types: &HashMap<u64, Vec<String>>) {
    if perform_for_types.is_empty() {
        return;
    }
    for item in &mut file.items {
        match item {
            lir::Item::Fn(f) => patch_expr_type_args(&mut f.value, perform_for_types),
            lir::Item::Impl(impl_decl) => {
                for m in &mut impl_decl.methods {
                    patch_expr_type_args(&mut m.value, perform_for_types);
                }
            }
            _ => {}
        }
    }
}

fn patch_expr_type_args(expr: &mut lir::Expr, perform_for_types: &HashMap<u64, Vec<String>>) {
    match expr {
        lir::Expr::Perform { id, type_args, .. } => {
            if let Some(resolved) = perform_for_types.get(&(id.0 as u64)) {
                *type_args = resolved.clone();
            }
        }
        lir::Expr::Handle { handler, body, .. } => {
            patch_expr_type_args(handler, perform_for_types);
            patch_expr_type_args(body, perform_for_types);
        }
        lir::Expr::Apply { callee, arg, .. } => {
            patch_expr_type_args(callee, perform_for_types);
            patch_expr_type_args(arg, perform_for_types);
        }
        lir::Expr::Let { value, body, .. } => {
            patch_expr_type_args(value, perform_for_types);
            patch_expr_type_args(body, perform_for_types);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            patch_expr_type_args(scrutinee, perform_for_types);
            for arm in arms {
                patch_expr_type_args(&mut arm.body, perform_for_types);
            }
        }
        lir::Expr::Lambda { body, .. } => patch_expr_type_args(body, perform_for_types),
        lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Force { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Ann { expr, .. } => patch_expr_type_args(expr, perform_for_types),
        lir::Expr::Member { object, .. } => patch_expr_type_args(object, perform_for_types),
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                patch_expr_type_args(&mut e.body, perform_for_types);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                patch_expr_type_args(a, perform_for_types);
            }
        }
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

/// Fill default type_args for Perform/Handle nodes and FnDecl/ExternFn cap annotations
/// with empty type_args. Sets type_args = [cap_name] so all caps consistently have self
/// as first type_arg.
fn fill_default_type_args(file: &mut lir::File) {
    for item in &mut file.items {
        match item {
            lir::Item::Fn(f) => {
                fill_cap_ref_default_type_args(&mut f.cap);
                fill_expr_default_type_args(&mut f.value);
            }
            lir::Item::ExternFn(f) => {
                fill_cap_ref_default_type_args(&mut f.cap);
            }
            lir::Item::Impl(impl_decl) => {
                for m in &mut impl_decl.methods {
                    fill_expr_default_type_args(&mut m.value);
                }
            }
            _ => {}
        }
    }
}

/// Fill default type_args in a CapRef's entries.
fn fill_cap_ref_default_type_args(cap: &mut Option<CapRef>) {
    let entries = match cap {
        Some(entries) => entries,
        None => return,
    };
    for entry in entries {
        if let CapEntry::Cap(TypeExpr::Cap { name, type_args }) = entry {
            if type_args.is_empty() {
                *type_args = vec![TypeExpr::Named(name.clone())];
            }
        }
    }
}

fn fill_expr_default_type_args(expr: &mut lir::Expr) {
    match expr {
        lir::Expr::Perform { cap, type_args, .. } => {
            if type_args.is_empty() {
                *type_args = vec![cap.clone()];
            }
        }
        lir::Expr::Handle { cap, type_args, handler, body, .. } => {
            if type_args.is_empty() {
                *type_args = vec![cap.clone()];
            }
            fill_expr_default_type_args(handler);
            fill_expr_default_type_args(body);
        }
        lir::Expr::Apply { callee, arg, .. } => {
            fill_expr_default_type_args(callee);
            fill_expr_default_type_args(arg);
        }
        lir::Expr::Let { value, body, .. } => {
            fill_expr_default_type_args(value);
            fill_expr_default_type_args(body);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            fill_expr_default_type_args(scrutinee);
            for arm in arms {
                fill_expr_default_type_args(&mut arm.body);
            }
        }
        lir::Expr::Lambda { body, .. } => fill_expr_default_type_args(body),
        lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Force { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Ann { expr, .. } => fill_expr_default_type_args(expr),
        lir::Expr::Member { object, .. } => fill_expr_default_type_args(object),
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                fill_expr_default_type_args(&mut e.body);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                fill_expr_default_type_args(a);
            }
        }
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}


impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}
