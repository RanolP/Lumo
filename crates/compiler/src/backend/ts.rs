use crate::{
    backend::{Backend, BackendError, BackendKind, CodegenTarget},
    lir::{self, AsRawValue},
    types::{CapRef, Pattern, TypeExpr},
};
use simple_ts_ast as tsast;
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct TypeScriptBackend;

struct LoweringContext {
    direct_callable_arities: HashMap<String, usize>,
    /// Maps function name → cap runtime param names (e.g. `__cap_Add_Number`).
    fn_caps: HashMap<String, Vec<String>>,
    /// Maps (impl_const_name, method_name) → arity for uncurrying member calls.
    impl_method_arities: HashMap<(String, String), usize>,
    /// Impl method bodies that use effects (Perform or calls to effectful fns).
    /// Entry present ⇒ method takes an extra `__caps` bundle + `__k` continuation
    /// and its body is CPS-compiled. Value is the set of cap runtime names
    /// treated as handled while CPS-lowering the body.
    impl_method_caps: HashMap<(String, String), Vec<String>>,
    /// Default impls available in the module: (cap_name, type_args) → impl const name.
    /// Populated from `impl Cap { ... }` (platform default) and `impl Type: Cap { ... }` (typeclass default).
    default_impls: HashMap<(String, Vec<String>), String>,
    /// All cap-decl names in the module. Used to classify impl blocks:
    /// `impl Cap { ... }` with target in this set is a platform default
    /// handler; `impl T: Cap { ... }` is a typeclass handler; everything
    /// else is an inherent impl for UFCS dispatch.
    cap_names: std::collections::HashSet<String>,
    /// Variants annotated with `#[as__raw(true|false)]`.
    /// Keyed by `(owner_type, variant_name)` for qualified Ctor references,
    /// and also by just `variant_name` for match patterns (which don't
    /// carry the owner type).
    ctor_as_raw: HashMap<(String, String), AsRawValue>,
    variant_as_raw: HashMap<String, AsRawValue>,
    match_counter: Cell<usize>,
    k_counter: Cell<usize>,
}

impl LoweringContext {
    fn next_match_name(&self) -> String {
        let n = self.match_counter.get();
        self.match_counter.set(n + 1);
        format!("__match_{n}")
    }

    fn next_k_name(&self) -> String {
        let n = self.k_counter.get();
        self.k_counter.set(n + 1);
        format!("__k_{n}")
    }
}

impl TypeScriptBackend {
    pub fn new() -> Self {
        Self
    }

    fn lower_file(&self, file: &lir::File) -> Result<tsast::Program, BackendError> {
        let mut body = Vec::new();
        let mut extern_names = HashMap::new();
        let fn_caps = collect_fn_caps(file);
        let impl_method_caps = collect_impl_method_caps(file, &fn_caps);
        let cap_names: std::collections::HashSet<String> = file
            .items
            .iter()
            .filter_map(|item| match item {
                lir::Item::Cap(c) => Some(c.name.clone()),
                _ => None,
            })
            .collect();
        let (ctor_as_raw, variant_as_raw) = collect_as_raw_variants(file);
        let ctx = LoweringContext {
            direct_callable_arities: collect_direct_callable_arities(file),
            fn_caps,
            impl_method_arities: collect_impl_method_arities(file),
            impl_method_caps,
            default_impls: collect_default_impls(file),
            cap_names,
            ctor_as_raw,
            variant_as_raw,
            match_counter: Cell::new(0),
            k_counter: Cell::new(0),
        };

        // Deduplicate extern types: prefer annotated over bare
        let mut deduped_extern_types: HashMap<String, &lir::ExternTypeDecl> = HashMap::new();
        for item in &file.items {
            if let lir::Item::ExternType(ext) = item {
                deduped_extern_types
                    .entry(ext.name.clone())
                    .and_modify(|existing| {
                        if ext.extern_name.is_some() {
                            *existing = ext;
                        }
                    })
                    .or_insert(ext);
            }
        }

        // Emit deduplicated extern types
        for ext in deduped_extern_types.values() {
            body.push(tsast::Stmt::TypeAlias(tsast::TypeAlias {
                export: true,
                name: ext.name.clone(),
                type_params: Vec::new(),
                ty: ts_type_from_extern_name(ext),
            }));
        }

        for item in &file.items {
            match item {
                lir::Item::ExternType(_) => {
                    // Already emitted above (deduplicated)
                }
                lir::Item::ExternFn(func) => {
                    let params = func
                        .params
                        .iter()
                        .map(|param| {
                            tsast::Param::new(&param.name)
                                .with_type(lower_type_expr_to_ts_type(&param.ty.value))
                        })
                        .collect::<Vec<_>>();
                    let unit_ty = TypeExpr::Named("Unit".to_owned());
                    let return_ty = func
                        .return_type
                        .as_ref()
                        .map(|s| &s.value)
                        .unwrap_or(&unit_ty);
                    let extern_path = func
                        .extern_name
                        .clone()
                        .unwrap_or_else(|| format!("globalThis.{}", func.name));
                    extern_names.insert(func.name.clone(), extern_path.clone());
                    let body_expr = extern_body_expr(
                        &extern_path,
                        &params,
                        return_ty,
                        bool_is_native_via_as_raw(&ctx.ctor_as_raw),
                    );
                    body.push(tsast::Stmt::Function(tsast::FunctionDecl {
                        export: true,
                        name: func.name.clone(),
                        type_params: Vec::new(),
                        params,
                        return_type: Some(lower_type_expr_to_ts_type(return_ty)),
                        body: tsast::FunctionBody::Expr(Box::new(body_expr)),
                        inline_always: func.inline,
                    }));
                }
                lir::Item::Data(data) => {
                    body.push(tsast::Stmt::TypeAlias(tsast::TypeAlias {
                        export: true,
                        name: data.name.clone(),
                        type_params: data.generics.clone(),
                        ty: lower_data_type(data),
                    }));
                    body.push(tsast::Stmt::Const(lower_data_bundle_const(data)));
                }
                lir::Item::Cap(cap) => {
                    // Emit a per-cap bundle type alias so effectful fn
                    // signatures can reference `__Bundle_<CapName>` with
                    // precise op signatures (no `any` / `unknown`).
                    body.push(tsast::Stmt::TypeAlias(emit_cap_bundle_alias(cap)));
                }
                lir::Item::Use(_) => {
                    // Use items produce no TS output
                }
                lir::Item::Fn(func) => {
                    body.push(tsast::Stmt::Function(lower_fn_decl(func, &ctx)?));
                    if func.name == "main" {
                        if let Some(wrapper) = emit_main_entry_wrapper(func, &ctx)? {
                            body.push(tsast::Stmt::Function(wrapper));
                        }
                    }
                }
                lir::Item::Impl(impl_decl) => {
                    body.push(tsast::Stmt::Const(lower_impl_const(impl_decl, &ctx)?));
                }
            }
        }

        let _ = extern_names;
        let mut program = tsast::Program::new(body);
        tsast::lower_expression_bodies(&mut program);
        tsast::inline_always_calls(&mut program);
        tsast::flatten_iifes(&mut program);
        tsast::return_lifting(&mut program);
        tsast::return_lifting(&mut program);
        tsast::flatten_iifes(&mut program); // catch IIFEs exposed by return_lifting
        tsast::collapse_let_to_const(&mut program);
        tsast::inline_single_use_consts(&mut program);
        tsast::inline_trivial_consts(&mut program);
        validate_program_has_no_any_or_unknown(&program)?;
        Ok(program)
    }
}

fn collect_direct_callable_arities(file: &lir::File) -> HashMap<String, usize> {
    let mut out = HashMap::new();
    for item in &file.items {
        match item {
            lir::Item::ExternFn(func) => {
                out.insert(func.name.clone(), func.params.len());
            }
            lir::Item::Fn(func) => {
                // Exclude effectful functions — they need CPS handling at call sites
                let is_effectful = func
                    .cap
                    .as_ref()
                    .map_or(false, |c| c.is_effectful());
                if !is_effectful {
                    out.insert(func.name.clone(), func.params.len());
                }
            }
            _ => {}
        }
    }
    out
}

fn collect_impl_method_arities(file: &lir::File) -> HashMap<(String, String), usize> {
    let mut out = HashMap::new();
    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            let const_name = impl_const_name(impl_decl);
            for method in &impl_decl.methods {
                out.insert(
                    (const_name.clone(), method.name.clone()),
                    method.params.len(),
                );
            }
        }
    }
    out
}

/// Build default impl map from all Impl items in the file.
/// Collect data variants annotated with `#[as__raw(true|false)]`.
/// Returns two maps for convenience:
/// - `ctor_as_raw` keyed by `(owner_type, variant_name)` — used at Ctor
///   construction sites where we know the owning type.
/// - `variant_as_raw` keyed by the bare `variant_name` — used at match
///   arms whose `Pattern::Ctor` carries only the short variant name.
fn collect_as_raw_variants(
    file: &lir::File,
) -> (
    HashMap<(String, String), AsRawValue>,
    HashMap<String, AsRawValue>,
) {
    let mut by_owner = HashMap::new();
    let mut by_variant = HashMap::new();
    for item in &file.items {
        if let lir::Item::Data(data) = item {
            for v in &data.variants {
                if let Some(raw) = v.as_raw.clone() {
                    by_owner.insert((data.name.clone(), v.name.clone()), raw.clone());
                    by_variant.insert(v.name.clone(), raw);
                }
            }
        }
    }
    (by_owner, by_variant)
}

fn raw_value_to_ts_expr(raw: &AsRawValue) -> tsast::Expr {
    match raw {
        AsRawValue::True => tsast::Expr::Bool(true),
        AsRawValue::False => tsast::Expr::Bool(false),
    }
}

/// - Platform default: `impl StrOps { ... }` where target matches a cap name → (cap, [cap]) → "StrOps"
/// - Typeclass default: `impl Number: Add { ... }` → (cap, [target]) → impl_const_name
fn collect_default_impls(file: &lir::File) -> HashMap<(String, Vec<String>), String> {
    let cap_names: std::collections::HashSet<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            lir::Item::Cap(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();

    let mut out = HashMap::new();
    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            let target = impl_decl.target_type.value.display();
            if impl_decl.capability.is_none() && cap_names.contains(&target) {
                out.insert((target.clone(), vec![target.clone()]), target);
            } else if let Some(cap_ty) = &impl_decl.capability {
                let cap = cap_ty.value.display();
                if cap_names.contains(&cap) {
                    let const_name = impl_decl
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("__impl_{target}_{cap}"));
                    out.insert((cap, vec![target]), const_name);
                }
            }
        }
    }
    out
}

/// Recursively collect cap runtime names required by an expression:
/// - any `Perform { cap, type_args }` contributes `cap_runtime_name(cap, type_args)`
/// - any call to a known effectful fn contributes that fn's caps
/// Stops descending into `Handle` for the cap it handles (that cap is consumed locally).
fn collect_expr_required_caps(
    expr: &lir::Expr,
    fn_caps: &HashMap<String, Vec<String>>,
    out: &mut Vec<String>,
) {
    match expr {
        lir::Expr::Perform { cap, type_args, .. } => {
            let name = cap_runtime_name(cap, type_args);
            if !out.contains(&name) {
                out.push(name);
            }
        }
        lir::Expr::Apply { callee, arg, .. } => {
            collect_expr_required_caps(callee, fn_caps, out);
            collect_expr_required_caps(arg, fn_caps, out);
        }
        lir::Expr::Force { expr, .. } => {
            if let lir::Expr::Ident { name, .. } = expr.as_ref() {
                if let Some(caps) = fn_caps.get(name) {
                    for c in caps {
                        if !out.contains(c) {
                            out.push(c.clone());
                        }
                    }
                }
            }
            collect_expr_required_caps(expr, fn_caps, out);
        }
        lir::Expr::Let { value, body, .. } => {
            collect_expr_required_caps(value, fn_caps, out);
            collect_expr_required_caps(body, fn_caps, out);
        }
        lir::Expr::Match { scrutinee, arms, .. } => {
            collect_expr_required_caps(scrutinee, fn_caps, out);
            for arm in arms {
                collect_expr_required_caps(&arm.body, fn_caps, out);
            }
        }
        lir::Expr::Lambda { body, .. } => collect_expr_required_caps(body, fn_caps, out),
        lir::Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            collect_expr_required_caps(handler, fn_caps, out);
            let handled = cap_runtime_name(cap, type_args);
            // Record body's caps but drop the one this Handle consumes locally.
            let mut body_caps = Vec::new();
            collect_expr_required_caps(body, fn_caps, &mut body_caps);
            for c in body_caps {
                if c != handled && !out.contains(&c) {
                    out.push(c);
                }
            }
        }
        lir::Expr::Thunk { expr, .. }
        | lir::Expr::Produce { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Ann { expr, .. } => collect_expr_required_caps(expr, fn_caps, out),
        lir::Expr::Bundle { entries, .. } => {
            for e in entries {
                collect_expr_required_caps(&e.body, fn_caps, out);
            }
        }
        lir::Expr::Ctor { args, .. } => {
            for a in args {
                collect_expr_required_caps(a, fn_caps, out);
            }
        }
        lir::Expr::Member { object, .. } => collect_expr_required_caps(object, fn_caps, out),
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Error { .. } => {}
    }
}

/// Compute which impl methods need CPS lowering.
///
/// A method needs CPS form if either:
/// - The impl provides a capability (either `impl Cap { ... }` platform default
///   where `target == cap_name`, or `impl T: Cap { ... }` typeclass default).
///   Cap methods are invoked through the `__caps` bundle under a uniform
///   calling convention `(args..., __caps, __k)`, so they must accept those
///   parameters even when the body is pure.
/// - The impl is inherent (`impl T { ... }` with no cap), but the body uses
///   a Perform or calls an effectful fn.
///
/// The returned `Vec<String>` lists cap runtime names that should be treated
/// as handled while CPS-lowering the body (empty for pure bodies).
fn collect_impl_method_caps(
    file: &lir::File,
    fn_caps: &HashMap<String, Vec<String>>,
) -> HashMap<(String, String), Vec<String>> {
    let cap_names: std::collections::HashSet<String> = file
        .items
        .iter()
        .filter_map(|item| match item {
            lir::Item::Cap(c) => Some(c.name.clone()),
            _ => None,
        })
        .collect();

    let mut out = HashMap::new();
    for item in &file.items {
        if let lir::Item::Impl(impl_decl) = item {
            let const_name = impl_const_name(impl_decl);
            let target = impl_decl.target_type.value.display();
            let is_cap_impl = impl_decl.capability.is_some()
                || (impl_decl.capability.is_none() && cap_names.contains(&target));
            for method in &impl_decl.methods {
                let mut caps = Vec::new();
                collect_expr_required_caps(&method.value, fn_caps, &mut caps);
                if is_cap_impl || !caps.is_empty() {
                    out.insert((const_name.clone(), method.name.clone()), caps);
                }
            }
        }
    }
    out
}

fn collect_fn_caps(file: &lir::File) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for item in &file.items {
        match item {
            lir::Item::ExternFn(func) => {
                let caps = func
                    .cap
                    .as_ref()
                    .map(|c| c.cap_mangled_params())
                    .unwrap_or_default();
                out.insert(func.name.clone(), caps);
            }
            lir::Item::Fn(func) => {
                let caps = func
                    .cap
                    .as_ref()
                    .map(|c| c.cap_mangled_params())
                    .unwrap_or_default();
                out.insert(func.name.clone(), caps);
            }
            _ => {}
        }
    }
    out
}

/// Build the runtime JS variable name for a cap with optional type args.
/// e.g. ("Add", ["Number"]) → "__cap_Add_Number", ("IO", []) → "__cap_IO"
///
/// Used as the key in `handled_caps` tracking. For code *emission*, use
/// `cap_bundle_access_expr` — caps are accessed through the `__caps` bundle
/// as `__caps.Add_Number`, not as bare `__cap_Add_Number` identifiers.
fn cap_runtime_name(cap: &str, type_args: &[String]) -> String {
    if type_args.is_empty() {
        format!("__cap_{cap}")
    } else {
        format!("__cap_{}_{}", cap, type_args.join("_"))
    }
}

/// Bundle property key for a cap: e.g. ("Add", ["Number"]) → "Add_Number".
/// Caps are grouped into a single `__caps` record per effectful function;
/// this helper computes the property name within that record.
fn cap_bundle_key(cap: &str, type_args: &[String]) -> String {
    if type_args.is_empty() {
        cap.to_owned()
    } else {
        format!("{}_{}", cap, type_args.join("_"))
    }
}

/// Strip the `__cap_` prefix from a runtime cap name to recover its bundle key.
fn cap_bundle_key_from_runtime(runtime_name: &str) -> &str {
    runtime_name.strip_prefix("__cap_").unwrap_or(runtime_name)
}

/// Parse a mangled runtime cap name back into `(cap_name, type_args)`.
/// Assumes cap/type names contain no underscores — all underscore-separated
/// segments after `__cap_` are split with the first being the cap.
fn cap_runtime_to_pair(runtime_name: &str) -> Option<(String, Vec<String>)> {
    let rest = runtime_name.strip_prefix("__cap_")?;
    let mut parts = rest.split('_');
    let cap = parts.next()?.to_owned();
    let type_args: Vec<String> = parts.map(|s| s.to_owned()).collect();
    Some((cap, type_args))
}

/// Expression accessing a cap from the ambient `__caps` bundle: `__caps.Key`.
fn cap_bundle_access_expr(cap: &str, type_args: &[String]) -> tsast::Expr {
    tsast::Expr::Member {
        object: Box::new(tsast::Expr::Ident("__caps".to_owned())),
        property: cap_bundle_key(cap, type_args),
    }
}

/// Same as `cap_bundle_access_expr`, but from a runtime mangled name.
fn cap_bundle_access_from_runtime(runtime_name: &str) -> tsast::Expr {
    tsast::Expr::Member {
        object: Box::new(tsast::Expr::Ident("__caps".to_owned())),
        property: cap_bundle_key_from_runtime(runtime_name).to_owned(),
    }
}

const CAPS_PARAM: &str = "__caps";

/// Wrap a CPS body in `__thunk(() => body)` so `__trampoline` can unwind
/// arbitrarily deep CPS call chains iteratively instead of blowing the stack.
fn thunk_wrap(body: tsast::Expr) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident("__thunk".to_owned())),
        args: vec![tsast::Expr::Arrow {
            params: Vec::new(),
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
        }],
    }
}

/// Whether `expr` is a call whose callee is already known to return a thunk,
/// meaning an enclosing `thunk_wrap` would be redundant. Used by Pass 2
/// (tail `__thunk` elision) to avoid double-bouncing the trampoline.
///
/// Returns true for:
/// - User-level effectful fn calls: `Call { callee: Ident(name), .. }` where
///   `fn_caps[name]` is non-empty (the callee's body is `__thunk`-wrapped).
/// - Cap-method dispatch: `Call { callee: Member { object: Ident("__caps"), .. }, .. }` —
///   cap-impl methods and handler bundle methods always CPS-wrap with `__thunk`.
/// - Effectful impl method calls: `Call { callee: Member { object: Ident(obj), property: method }, .. }`
///   where `(obj, method)` is registered in `impl_method_caps`.
///
/// Conservative by default: anything else (including `__k(...)` and pure
/// extern calls) returns false, preserving the outer `__thunk` wrap.
fn is_thunk_returning_call(expr: &tsast::Expr, ctx: &LoweringContext) -> bool {
    let tsast::Expr::Call { callee, .. } = expr else {
        return false;
    };
    match callee.as_ref() {
        tsast::Expr::Ident(name) => ctx
            .fn_caps
            .get(name)
            .map_or(false, |caps| !caps.is_empty()),
        tsast::Expr::Member { object, property } => match object.as_ref() {
            tsast::Expr::Ident(obj) if obj == CAPS_PARAM => true,
            tsast::Expr::Ident(obj) => ctx
                .impl_method_caps
                .contains_key(&(obj.clone(), property.clone())),
            _ => false,
        },
        _ => false,
    }
}

/// The shared identity continuation reference `__identity`. Runtime prelude
/// emits `const __identity = (__v) => __v;` once, and every CPS entry point
/// that would otherwise build a fresh `(__v) => __v` closure references this
/// const instead. Saves one closure allocation per top-level handle / main
/// invocation.
fn identity_k_expr() -> tsast::Expr {
    tsast::Expr::Ident("__identity".to_owned())
}

/// Fallback TS type for the ambient capabilities bundle when a precise
/// inline bundle shape isn't known. Emitted as the structural
/// `{ [key: string]: { [op: string]: __CpsFn } }` alias in the runtime
/// prelude. Prefer `caps_bundle_type(&CapRef)` at every call site that has
/// the function's cap annotation available.
fn caps_type() -> tsast::TsType {
    tsast::TsType::TypeRef("__Caps".to_owned())
}

/// Precise `__caps` bundle type built from a function's cap annotation.
/// Emits an inline structural type `{ readonly Key1: __Bundle_Cap1<T1>; ... }`
/// so each key is individually typed against its specific bundle interface.
fn caps_bundle_type(cap_ref: &CapRef) -> tsast::TsType {
    let entries = cap_ref.entries();
    if entries.is_empty() {
        return caps_type();
    }
    let mut parts: Vec<String> = Vec::new();
    for entry in entries {
        match entry {
            TypeExpr::Cap { name, type_args } => {
                let type_arg_names: Vec<String> =
                    type_args.iter().map(|t| t.display()).collect();
                let key = cap_bundle_key(name, &type_arg_names);
                let type_arg_str = if type_args.is_empty() {
                    String::new()
                } else {
                    let rendered: Vec<String> =
                        type_args.iter().map(type_expr_to_ts_text).collect();
                    format!("<{}>", rendered.join(", "))
                };
                parts.push(format!(
                    "readonly {}: __Bundle_{}{}",
                    key, name, type_arg_str
                ));
            }
            TypeExpr::Named(name) => {
                // Platform default keyed as (cap, [cap]) — e.g. NumOps → `NumOps_NumOps`.
                let key = cap_bundle_key(name, &[name.clone()]);
                parts.push(format!("readonly {}: __Bundle_{}", key, name));
            }
            _ => return caps_type(),
        }
    }
    tsast::TsType::Raw(format!("{{ {} }}", parts.join("; ")))
}

/// Precise continuation type: `__Kont<ReturnType>` using the fn's declared
/// return. Unit maps to `void` (TS allows `__Kont<void>`).
fn kont_type(return_type: Option<&TypeExpr>) -> tsast::TsType {
    let ret_text = return_type.map_or_else(|| "void".to_owned(), type_expr_to_ts_text);
    tsast::TsType::Raw(format!("__Kont<{}>", ret_text))
}

/// Continuation type when the value's concrete type isn't statically known
/// at this site. Uses `__Kont<__CpsValue>`.
fn kont_type_any() -> tsast::TsType {
    tsast::TsType::Raw("__Kont<__CpsValue>".to_owned())
}

/// TS type for every effectful function's return: `__Ret`, a union of
/// `__Thunk` (for trampoline bouncing) and concrete runtime values.
fn ret_type() -> tsast::TsType {
    tsast::TsType::TypeRef("__Ret".to_owned())
}

/// Whether any of this cap's operation signatures refer to the `Self`
/// placeholder — if so, the emitted bundle alias takes a `Self` type param.
fn cap_uses_self(cap: &lir::CapDecl) -> bool {
    cap.operations.iter().any(|op| {
        op.params.iter().any(|p| type_refs_self(&p.ty.value))
            || op.return_type
                .as_ref()
                .map_or(false, |r| type_refs_self(&r.value))
    })
}

fn type_refs_self(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Named(n) => n == "Self",
        TypeExpr::App { head, args } => head == "Self" || args.iter().any(type_refs_self),
        TypeExpr::Produce(inner) | TypeExpr::Thunk(inner) => type_refs_self(inner),
        TypeExpr::Cap { type_args, .. } => type_args.iter().any(type_refs_self),
    }
}

/// Emit a per-cap bundle alias:
/// `type __Bundle_Add<Self> = { readonly add: (__caps: __Caps, a: Self, b: Self, __k: __Kont<Self>) => __Ret; ... };`
/// Op signatures mirror the cap's operations exactly, with `__caps` and `__k`
/// added to match the runtime calling convention.
fn emit_cap_bundle_alias(cap: &lir::CapDecl) -> tsast::TypeAlias {
    let uses_self = cap_uses_self(cap);
    let type_params = if uses_self {
        vec!["Self".to_owned()]
    } else {
        Vec::new()
    };

    let op_entries: Vec<String> = cap
        .operations
        .iter()
        .map(|op| {
            let mut method_params: Vec<String> = vec!["__caps: __Caps".to_owned()];
            for p in &op.params {
                method_params.push(format!(
                    "{}: {}",
                    p.name,
                    type_expr_to_ts_text(&p.ty.value)
                ));
            }
            let ret_text = op
                .return_type
                .as_ref()
                .map_or_else(|| "void".to_owned(), |r| type_expr_to_ts_text(&r.value));
            method_params.push(format!("__k: __Kont<{}>", ret_text));
            format!(
                "readonly {}: ({}) => __Ret",
                op.name,
                method_params.join(", ")
            )
        })
        .collect();

    let body = if op_entries.is_empty() {
        "{}".to_owned()
    } else {
        format!("{{ {} }}", op_entries.join("; "))
    };

    tsast::TypeAlias {
        export: false,
        name: format!("__Bundle_{}", cap.name),
        type_params,
        ty: tsast::TsType::Raw(body),
    }
}

fn lower_fn_decl(
    func: &lir::FnDecl,
    ctx: &LoweringContext,
) -> Result<tsast::FunctionDecl, BackendError> {
    let (lowered_params, lowered_body) = unwrap_fn_value(&func.value)?;
    if lowered_params.len() != func.params.len() {
        return Err(BackendError::EmitFailed(format!(
            "function `{}` lowered to {} lambda params but signature has {} params",
            func.name,
            lowered_params.len(),
            func.params.len()
        )));
    }

    let user_params: Vec<tsast::Param> = func
        .params
        .iter()
        .zip(lowered_params.iter())
        .map(|(param, lowered_name)| {
            tsast::Param::new(lowered_name).with_type(lower_type_expr_to_ts_type(&param.ty.value))
        })
        .collect();

    let caps: Vec<String> = func
        .cap
        .as_ref()
        .map(|c| c.cap_mangled_params())
        .unwrap_or_default();

    let mut params: Vec<tsast::Param> = Vec::new();
    if !caps.is_empty() {
        // Spec: capability bundle is the FIRST parameter of every effectful
        // function. Signature: `fn(__caps, user_params..., __k)`. Types are
        // as precise as possible — `__caps` is an inline bundle with the
        // exact keys the function needs, `__k` is `__Kont<ReturnType>`.
        let caps_ty = func
            .cap
            .as_ref()
            .map_or_else(caps_type, caps_bundle_type);
        let return_ty_expr = func.return_type.as_ref().map(|s| &s.value);
        params.push(tsast::Param::new(CAPS_PARAM).with_type(caps_ty));
        params.extend(user_params);
        params.push(tsast::Param::new("__k").with_type(kont_type(return_ty_expr)));
        let raw_cps_body = lower_cps_expr(
            lowered_body,
            tsast::Expr::Ident("__k".to_owned()),
            &caps,
            ctx,
        );
        // Wrap the body in `__thunk(() => body)` so recursive CPS calls bounce
        // off `__trampoline` instead of growing the real call stack. Pass 2
        // elides the wrap when the body is a tail call to a callee that
        // already returns a thunk (passthrough functions).
        let cps_body = if is_thunk_returning_call(&raw_cps_body, ctx) {
            raw_cps_body
        } else {
            thunk_wrap(raw_cps_body)
        };
        // main() is the program entry point; callers invoke it as `main()` with
        // no args. If main is effectful, emit under a private name. The public
        // `main()` wrapper is emitted separately by `emit_main_entry_wrapper`.
        let is_main_entry = func.name == "main";
        let emitted_name = if is_main_entry {
            "__main_cps".to_string()
        } else {
            func.name.clone()
        };
        Ok(tsast::FunctionDecl {
            export: !is_main_entry,
            name: emitted_name,
            type_params: func.generics.clone(),
            params,
            return_type: Some(ret_type()),
            body: tsast::FunctionBody::Expr(Box::new(cps_body)),
            // Don't inline CPS-transformed functions — params and continuation
            // injection make naive substitution unsafe.
            inline_always: false,
        })
    } else {
        let unit_ty = TypeExpr::Named("Unit".to_owned());
        let return_ty = func
            .return_type
            .as_ref()
            .map(|s| &s.value)
            .unwrap_or(&unit_ty);
        Ok(tsast::FunctionDecl {
            export: true,
            name: func.name.clone(),
            type_params: func.generics.clone(),
            params: user_params,
            return_type: Some(lower_type_expr_to_ts_type(return_ty)),
            body: tsast::FunctionBody::Expr(Box::new(lower_expr(lowered_body, ctx))),
            inline_always: func.inline,
        })
    }
}

/// Emit the public `main()` wrapper when user's main is effectful.
/// The wrapper provides default cap impls for each required cap and an identity
/// continuation, then delegates to `__main_cps`. Returns `Ok(None)` if main is
/// pure (no wrapper needed). Returns `Err` if any required cap has no default impl.
fn emit_main_entry_wrapper(
    func: &lir::FnDecl,
    ctx: &LoweringContext,
) -> Result<Option<tsast::FunctionDecl>, BackendError> {
    let cap_ref = match &func.cap {
        Some(c) if c.is_effectful() => c,
        _ => return Ok(None),
    };

    // Collect main's directly required (cap, type_args) pairs, then expand
    // transitively: every default impl we resolve may itself need caps (via
    // `impl_method_caps`). Gather the closure so main's `__caps` bundle has
    // everything any callee will look up.
    let mut required: Vec<(String, Vec<String>)> = Vec::new();
    let mut pending: Vec<(String, Vec<String>)> = Vec::new();
    for entry in cap_ref.entries() {
        let pair = match entry {
            TypeExpr::Cap { name, type_args } => (
                name.clone(),
                type_args.iter().map(|t| t.display()).collect(),
            ),
            TypeExpr::Named(name) => (name.clone(), vec![name.clone()]),
            _ => {
                return Err(BackendError::EmitFailed(format!(
                    "main() requires non-cap capability: {entry:?}"
                )));
            }
        };
        if !required.contains(&pair) {
            required.push(pair.clone());
            pending.push(pair);
        }
    }

    let mut bundle_props: Vec<tsast::ObjectProp> = Vec::new();
    while let Some((cap_name, type_args)) = pending.pop() {
        // Platform default: `impl Cap { ... }` is keyed (cap, [cap]).
        let platform_key = (cap_name.clone(), vec![cap_name.clone()]);
        // Typeclass default: `impl T: Cap { ... }` is keyed (cap, [T]).
        let typeclass_key = (cap_name.clone(), type_args.clone());

        let impl_const = ctx
            .default_impls
            .get(&typeclass_key)
            .or_else(|| ctx.default_impls.get(&platform_key))
            .cloned()
            .ok_or_else(|| {
                let type_args_str = if type_args.is_empty() {
                    String::new()
                } else {
                    format!("[{}]", type_args.join(", "))
                };
                BackendError::EmitFailed(format!(
                    "main() requires capability `{cap_name}{type_args_str}` but no default impl is available — \
                     provide `impl {cap_name} {{ ... }}` (platform default) or `impl <T>: {cap_name} {{ ... }}` \
                     (typeclass default), or add an explicit `handle` block"
                ))
            })?;

        // Pull in any cap this impl's methods need (transitive closure).
        for ((const_name, _method), method_caps) in &ctx.impl_method_caps {
            if const_name != &impl_const {
                continue;
            }
            for runtime in method_caps {
                if let Some((cap_n, cap_a)) = cap_runtime_to_pair(runtime) {
                    if !required.iter().any(|p| p == &(cap_n.clone(), cap_a.clone())) {
                        required.push((cap_n.clone(), cap_a.clone()));
                        pending.push((cap_n, cap_a));
                    }
                }
            }
        }

        // Each default impl is now emitted as a handler factory
        // `(__k_handle) => { ops }`. Install it by invoking the factory with
        // `__identity` — at the outermost handle in main, a matching
        // perform's "rest of computation" trivially returns its own value.
        let installed = tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Ident(impl_const)),
            args: vec![identity_k_expr()],
        };
        bundle_props.push(tsast::ObjectProp {
            key: tsast::ObjectKey::Ident(cap_bundle_key(&cap_name, &type_args)),
            value: installed,
        });
    }

    // Body: `return __trampoline(__main_cps({ <bundle> }, __identity));`
    let call = tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident("__main_cps".to_owned())),
        args: vec![tsast::Expr::Object(bundle_props), identity_k_expr()],
    };
    let wrapped = tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident("__trampoline".to_owned())),
        args: vec![call],
    };

    Ok(Some(tsast::FunctionDecl {
        export: true,
        name: "main".to_owned(),
        type_params: Vec::new(),
        params: Vec::new(),
        return_type: Some(tsast::TsType::Void),
        body: tsast::FunctionBody::Expr(Box::new(wrapped)),
        inline_always: false,
    }))
}

fn lower_impl_const(
    impl_decl: &lir::ImplDecl,
    ctx: &LoweringContext,
) -> Result<tsast::ConstDecl, BackendError> {
    let const_name = impl_const_name(impl_decl);
    // Classify: is this impl a handler bundle (cap impl) that gets installed
    // via an implicit/explicit `handle`, or an inherent impl providing UFCS
    // dispatch on a value type?
    //
    // - `impl T: Cap { ... }` (typeclass default)      → cap impl
    // - `impl Cap { ... }` where target matches a cap   → platform default
    // - `impl T { ... }` where T is a non-cap type      → inherent (UFCS)
    //
    // Cap impls go through the handler-factory path (abort-by-default,
    // explicit `resume`); inherent impls keep the tail-CPS resume-via-__k
    // shape for regular effectful method calls.
    let target = impl_decl.target_type.value.display();
    let is_cap_impl = impl_decl.capability.is_some()
        || (impl_decl.capability.is_none() && ctx.cap_names.contains(&target));

    if is_cap_impl {
        return lower_cap_impl_const(impl_decl, &const_name, ctx);
    }

    lower_inherent_impl_const(impl_decl, &const_name, ctx)
}

/// Lower a `impl Cap { ... }` / `impl T: Cap { ... }` block as a handler
/// factory: `(__k_handle) => { op: (__caps, args..., __k_perform) => ... }`.
/// Semantics match user `handle Cap with <bundle> in body` — abort-by-default.
fn lower_cap_impl_const(
    impl_decl: &lir::ImplDecl,
    const_name: &str,
    ctx: &LoweringContext,
) -> Result<tsast::ConstDecl, BackendError> {
    let mut props: Vec<tsast::ObjectProp> = Vec::new();
    for method in &impl_decl.methods {
        let (_, lowered_body) = unwrap_fn_value(&method.value)?;
        let method_key = (const_name.to_owned(), method.name.clone());
        let empty: Vec<String> = Vec::new();
        let handled_caps = ctx
            .impl_method_caps
            .get(&method_key)
            .unwrap_or(&empty)
            .as_slice();
        props.push(emit_handler_method_prop(
            &method.name,
            &method.params,
            lowered_body,
            handled_caps,
            ctx,
        ));
    }
    Ok(tsast::ConstDecl {
        export: true,
        name: const_name.to_owned(),
        type_ann: None,
        init: emit_handler_factory(props),
    })
}

/// Lower a `impl T { ... }` inherent block as a plain method record.
/// Used for UFCS dispatch (`"hi".len()` → `String.len("hi")`); NOT a
/// handler. Effectful methods CPS-thread the tail value through `__k`
/// (implicit resume, i.e. normal effectful-function return semantics).
fn lower_inherent_impl_const(
    impl_decl: &lir::ImplDecl,
    const_name: &str,
    ctx: &LoweringContext,
) -> Result<tsast::ConstDecl, BackendError> {
    let mut properties = Vec::new();
    for method in &impl_decl.methods {
        let (lowered_params, lowered_body) = unwrap_fn_value(&method.value)?;
        if lowered_params.len() != method.params.len() {
            return Err(BackendError::EmitFailed(format!(
                "impl method `{}` lowered to {} lambda params but signature has {} params",
                method.name,
                lowered_params.len(),
                method.params.len()
            )));
        }
        let user_params: Vec<tsast::Param> = method
            .params
            .iter()
            .zip(lowered_params.iter())
            .map(|(param, lowered_name)| {
                tsast::Param::new(lowered_name)
                    .with_type(lower_type_expr_to_ts_type(&param.ty.value))
            })
            .collect();
        let unit_ty = TypeExpr::Named("Unit".to_owned());
        let return_ty = method
            .return_type
            .as_ref()
            .map(|s| &s.value)
            .unwrap_or(&unit_ty);

        let method_key = (const_name.to_owned(), method.name.clone());
        let is_effectful_method = ctx.impl_method_caps.contains_key(&method_key);
        let (params, body_expr) = if let Some(caps) = ctx.impl_method_caps.get(&method_key) {
            // Effectful inherent method: regular CPS-form fn — tail value
            // flows through `__k`, acting as an implicit resume.
            let mut params = vec![tsast::Param::new(CAPS_PARAM).with_type(caps_type())];
            params.extend(user_params);
            let method_ret = method.return_type.as_ref().map(|s| &s.value);
            params.push(tsast::Param::new("__k").with_type(kont_type(method_ret)));
            let body = lower_cps_expr(
                lowered_body,
                tsast::Expr::Ident("__k".to_owned()),
                caps,
                ctx,
            );
            let wrapped = if is_thunk_returning_call(&body, ctx) {
                body
            } else {
                thunk_wrap(body)
            };
            (params, wrapped)
        } else {
            (user_params, lower_expr(lowered_body, ctx))
        };

        let method_return_type = if is_effectful_method {
            ret_type()
        } else {
            lower_type_expr_to_ts_type(return_ty)
        };
        properties.push(tsast::ObjectProp {
            key: tsast::ObjectKey::Ident(method.name.clone()),
            value: tsast::Expr::Arrow {
                params,
                return_type: Some(method_return_type),
                body: Box::new(tsast::FunctionBody::Expr(Box::new(body_expr))),
            },
        });
    }

    Ok(tsast::ConstDecl {
        export: true,
        name: const_name.to_owned(),
        type_ann: None,
        init: tsast::Expr::Object(properties),
    })
}

/// Compute the JS/TS const name for an impl block.
fn impl_const_name(impl_decl: &lir::ImplDecl) -> String {
    if let Some(name) = &impl_decl.name {
        // Named impl: use the given name
        name.clone()
    } else if let Some(cap) = &impl_decl.capability {
        // Unnamed cap impl: __impl_{Target}_{Cap}
        let target = impl_decl.target_type.value.display();
        let cap = cap.value.display();
        format!("__impl_{target}_{cap}")
    } else {
        // Inherent impl: name after target type
        impl_decl.target_type.value.display()
    }
}

fn unwrap_fn_value(value: &lir::Expr) -> Result<(Vec<String>, &lir::Expr), BackendError> {
    let lir::Expr::Thunk { expr, .. } = value else {
        return Err(BackendError::EmitFailed(
            "lowered function value must start with thunk".to_owned(),
        ));
    };

    let mut params = Vec::new();
    let mut cursor = expr.as_ref();
    while let lir::Expr::Lambda { param, body, .. } = cursor {
        params.push(param.clone());
        cursor = body.as_ref();
    }
    Ok((params, cursor))
}

impl Backend for TypeScriptBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::TypeScript
    }

    fn supports(&self, target: CodegenTarget) -> bool {
        matches!(
            target,
            CodegenTarget::TypeScript
                | CodegenTarget::TypeScriptDefinition
                | CodegenTarget::JavaScript
        )
    }

    fn emit(&self, file: &lir::File, target: CodegenTarget) -> Result<String, BackendError> {
        let program = self.lower_file(file)?;
        let target = match target {
            CodegenTarget::TypeScript => tsast::EmitTarget::TypeScript,
            CodegenTarget::TypeScriptDefinition => tsast::EmitTarget::TypeScriptDefinition,
            CodegenTarget::JavaScript => tsast::EmitTarget::JavaScript,
            _ => return Err(BackendError::UnsupportedTarget(target)),
        };

        let emitted = tsast::Emitter::default().emit_program(&program, target);
        let imports = format_imports(file, target);
        let prelude = runtime_prelude(target, &emitted);
        Ok(format!("{prelude}{imports}{emitted}"))
    }
}

/// Collect `#[link(module = ...)]` extern fns and emit `import { ... } from "..."` statements.
fn format_imports(file: &lir::File, target: tsast::EmitTarget) -> String {
    if matches!(target, tsast::EmitTarget::TypeScriptDefinition) {
        return String::new();
    }
    // Group by module, preserving insertion order.
    let mut modules: Vec<(String, Vec<(String, String)>)> = Vec::new();
    let mut seen_per_module: HashMap<String, std::collections::HashSet<(String, String)>> =
        HashMap::new();
    for item in &file.items {
        if let lir::Item::ExternFn(func) = item {
            if let Some((module, js_name)) = &func.link_module {
                let alias = format!("__lumo_{}", func.name);
                let entry = (js_name.clone(), alias);
                let seen = seen_per_module.entry(module.clone()).or_default();
                if seen.insert(entry.clone()) {
                    if let Some((_, names)) = modules.iter_mut().find(|(m, _)| m == module) {
                        names.push(entry);
                    } else {
                        modules.push((module.clone(), vec![entry]));
                    }
                }
            }
        }
    }
    if modules.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (module, names) in modules {
        let specs = names
            .iter()
            .map(|(js, alias)| {
                if js == alias {
                    js.clone()
                } else {
                    format!("{js} as {alias}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("import {{ {specs} }} from \"{module}\";\n"));
    }
    out.push('\n');
    out
}

fn runtime_prelude(target: tsast::EmitTarget, emitted: &str) -> String {
    if matches!(target, tsast::EmitTarget::TypeScriptDefinition) {
        return String::new();
    }
    let needs_trampoline = emitted.contains("__trampoline(") || emitted.contains("__thunk(");
    let needs_match_error = emitted.contains("__lumo_match_error(");
    let needs_error = emitted.contains("__lumo_error(");
    let needs_identity = emitted.contains("__identity");
    let ts = matches!(target, tsast::EmitTarget::TypeScript);

    let mut out = String::new();
    out.push_str("const LUMO_TAG = Symbol.for(\"Lumo/tag\");\n");
    if ts {
        out.push_str("type __LumoRuntime = { [LUMO_TAG]: string; args?: __LumoRuntime[] } | string | number | boolean | null | undefined | (() => __LumoRuntime);\n");
    }
    if needs_match_error {
        if ts {
            out.push_str("const __lumo_match_error = (value: __LumoRuntime): never => { throw new Error(\"non-exhaustive match: \" + JSON.stringify(value)); };\n");
        } else {
            out.push_str("const __lumo_match_error = (value) => { throw new Error(\"non-exhaustive match: \" + JSON.stringify(value)); };\n");
        }
    }
    if needs_error {
        if ts {
            out.push_str("const __lumo_error = (): never => { throw new Error(\"lumo runtime error\"); };\n");
        } else {
            out.push_str("const __lumo_error = () => { throw new Error(\"lumo runtime error\"); };\n");
        }
    }
    if needs_trampoline {
        if ts {
            // Note: __thunk and __trampoline are declared *after* the CPS
            // types (__Thunk / __Ret / __CpsValue) since they reference them.
        } else {
            out.push_str("const __thunk = (fn) => { fn.__t = 1; return fn; };\n");
            out.push_str("const __trampoline = (v) => { while (v && v.__t) v = v(); return v; };\n");
        }
    }
    // CPS-plumbing type aliases. No `any` / `unknown`. `__CpsValue` is a
    // recursive union covering every runtime value (primitives, data variants,
    // bundles, functions). `__Thunk` is a trampoline-flagged thunk; `__Ret`
    // is what an effectful function returns (either a thunk to bounce, or a
    // concrete value). `__Kont<T>` is a continuation taking a T and
    // returning any `__Ret`. `__Caps` is the structural shape used as a
    // fallback when a specific bundle type isn't known at a call site.
    let needs_cps_types = emitted.contains("__Ret")
        || emitted.contains("__Kont")
        || emitted.contains("__Caps")
        || emitted.contains("__CpsValue")
        || emitted.contains("__Thunk")
        || emitted.contains("__thunk(")
        || emitted.contains("__trampoline(");
    if ts && needs_cps_types {
        out.push_str(
            "type __CpsFn = (...args: readonly __CpsValue[]) => __Ret;\n",
        );
        out.push_str(
            "type __CpsValue = __LumoRuntime | __CpsFn | { readonly [key: string]: __CpsValue };\n",
        );
        out.push_str("type __Thunk = { (): __Ret; __t: 1 };\n");
        out.push_str("type __Ret = __Thunk | __CpsValue;\n");
        out.push_str("type __Kont<T = __CpsValue> = (__v: T) => __Ret;\n");
        out.push_str(
            "type __Caps = { readonly [key: string]: { readonly [op: string]: __CpsFn } };\n",
        );
    }
    if ts && needs_trampoline {
        out.push_str("const __thunk = (fn: () => __Ret): __Thunk => { (fn as __Thunk).__t = 1; return fn as __Thunk; };\n");
        out.push_str(
            "const __trampoline = (v: __Ret): __CpsValue => { \
             let cur: __Ret = v; \
             while (typeof cur === \"function\" && (cur as __Thunk).__t === 1) { cur = (cur as __Thunk)(); } \
             return cur as __CpsValue; };\n",
        );
    }
    if needs_identity {
        if ts {
            out.push_str("const __identity = <T>(__v: T): T => __v;\n");
        } else {
            out.push_str("const __identity = (__v) => __v;\n");
        }
    }
    out.push('\n');
    out
}

fn lower_type_expr_to_ts_type(ty: &TypeExpr) -> tsast::TsType {
    match ty {
        TypeExpr::Named(name) if name == "Unit" => tsast::TsType::Void,
        // Parser fills missing annotations with the `<missing>` sentinel;
        // that's not a valid TS identifier. Fall back to the CPS-plumbing
        // union `__CpsValue` so the emitted signature still type-checks.
        TypeExpr::Named(name) if name == "<missing>" => {
            tsast::TsType::TypeRef("__CpsValue".to_owned())
        }
        TypeExpr::Named(name) => tsast::TsType::TypeRef(name.clone()),
        TypeExpr::App { head, args } => tsast::TsType::TypeRef(format!(
            "{head}<{}>",
            args.iter()
                .map(type_expr_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        )),
        // `produce T` → just the inner type in TS
        TypeExpr::Produce(inner) => lower_type_expr_to_ts_type(inner),
        // `thunk T` → `() => T` in TS
        TypeExpr::Thunk(inner) => lower_type_expr_to_ts_type(inner),
        TypeExpr::Cap { name, .. } => tsast::TsType::TypeRef(name.clone()),
    }
}

fn lower_data_type(data: &lir::DataDecl) -> tsast::TsType {
    if data.variants.is_empty() {
        return tsast::TsType::Never;
    }
    let union = data
        .variants
        .iter()
        .map(|variant| {
            if let Some(raw) = &variant.as_raw {
                return match raw {
                    AsRawValue::True => "true".to_owned(),
                    AsRawValue::False => "false".to_owned(),
                };
            }
            if variant.payload.is_empty() {
                format!("{{ [LUMO_TAG]: '{}' }}", variant.name)
            } else {
                let payloads = variant
                    .payload
                    .iter()
                    .map(|payload| type_expr_to_ts_text(&payload.value))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ [LUMO_TAG]: '{}', args: [{}] }}", variant.name, payloads)
            }
        })
        .collect::<Vec<_>>()
        .join(" | ");
    tsast::TsType::Raw(union)
}

fn ts_type_from_extern_name(ext: &lir::ExternTypeDecl) -> tsast::TsType {
    match ext.extern_name.as_deref().unwrap_or(&ext.name) {
        "string" => tsast::TsType::String,
        "number" => tsast::TsType::Number,
        "boolean" => tsast::TsType::Boolean,
        "void" => tsast::TsType::Void,
        "null" => tsast::TsType::Null,
        "undefined" => tsast::TsType::Undefined,
        "never" => tsast::TsType::Never,
        other => tsast::TsType::TypeRef(other.to_owned()),
    }
}

fn expr_from_extern_path(path: &str) -> tsast::Expr {
    let mut parts = path.split('.').filter(|s| !s.is_empty());
    let Some(head) = parts.next() else {
        return tsast::Expr::Ident("<invalid-extern>".to_owned());
    };
    let mut acc = if is_js_ident(head) {
        tsast::Expr::Ident(head.to_owned())
    } else {
        tsast::Expr::Index {
            object: Box::new(tsast::Expr::Ident("globalThis".to_owned())),
            index: Box::new(tsast::Expr::String(head.to_owned())),
        }
    };
    for part in parts {
        acc = if is_js_ident(part) {
            tsast::Expr::Member {
                object: Box::new(acc),
                property: part.to_owned(),
            }
        } else {
            tsast::Expr::Index {
                object: Box::new(acc),
                index: Box::new(tsast::Expr::String(part.to_owned())),
            }
        };
    }
    acc
}

fn is_js_ident(text: &str) -> bool {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

/// Build the body expression for an extern fn from its JS path.
///
/// Rules:
/// - Operator externs (`_+_`, `_===_`, `-_`, etc.) use operator specialization.
/// - If path ends with `()`, it's a function call; otherwise a value/property access.
/// - If path contains `.prototype.`, the first param is the receiver (`this`) and
///   the rest are passed as args. Otherwise the path is used directly.
/// - If the return type is `Bool`, the call result is auto-wrapped in the Lumo Bool ADT.
fn extern_body_expr(
    extern_path: &str,
    params: &[tsast::Param],
    return_type: &TypeExpr,
    bool_is_native: bool,
) -> tsast::Expr {
    if let Some(expr) = specialize_operator_wrapper_expr(extern_path, params) {
        return maybe_bool_wrap(expr, return_type, bool_is_native);
    }

    let (base, is_call) = match extern_path.strip_suffix("()") {
        Some(base) => (base, true),
        None => (extern_path, false),
    };

    let expr = if let Some(idx) = base.find(".prototype.") {
        let member_path = &base[idx + ".prototype.".len()..];
        let receiver = tsast::Expr::Ident(params[0].name.clone());
        let target = member_access_chain(receiver, member_path);
        if is_call {
            let rest_args = params[1..]
                .iter()
                .map(|p| tsast::Expr::Ident(p.name.clone()))
                .collect();
            tsast::Expr::Call {
                callee: Box::new(target),
                args: rest_args,
            }
        } else {
            target
        }
    } else {
        let path_expr = expr_from_extern_path(base);
        if is_call {
            let args = params
                .iter()
                .map(|p| tsast::Expr::Ident(p.name.clone()))
                .collect();
            tsast::Expr::Call {
                callee: Box::new(path_expr),
                args,
            }
        } else {
            path_expr
        }
    };

    maybe_bool_wrap(expr, return_type, bool_is_native)
}

fn member_access_chain(mut acc: tsast::Expr, path: &str) -> tsast::Expr {
    for part in path.split('.').filter(|s| !s.is_empty()) {
        acc = if is_js_ident(part) {
            tsast::Expr::Member {
                object: Box::new(acc),
                property: part.to_owned(),
            }
        } else {
            tsast::Expr::Index {
                object: Box::new(acc),
                index: Box::new(tsast::Expr::String(part.to_owned())),
            }
        };
    }
    acc
}

fn maybe_bool_wrap(expr: tsast::Expr, return_type: &TypeExpr, bool_is_native: bool) -> tsast::Expr {
    if is_bool_return_type(return_type) {
        // When Bool is `#[as__raw(true)]`/`#[as__raw(false)]`-mapped, the
        // Lumo-level ctor values ARE the raw JS booleans — so the
        // `if cond then Bool.true else Bool.false` wrap is identity.
        if bool_is_native {
            expr
        } else {
            bool_wrap_expr(expr)
        }
    } else {
        expr
    }
}

/// True when `Bool.true` has `#[as__raw(true)]` and `Bool.false` has
/// `#[as__raw(false)]` — the shape that lets raw JS booleans from extern
/// calls flow through without a Bool-wrap.
fn bool_is_native_via_as_raw(ctor_as_raw: &HashMap<(String, String), AsRawValue>) -> bool {
    matches!(
        ctor_as_raw.get(&("Bool".to_owned(), "true".to_owned())),
        Some(AsRawValue::True)
    ) && matches!(
        ctor_as_raw.get(&("Bool".to_owned(), "false".to_owned())),
        Some(AsRawValue::False)
    )
}

fn is_bool_return_type(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Named(n) => n == "Bool",
        TypeExpr::Produce(inner) => is_bool_return_type(inner),
        _ => false,
    }
}

fn bool_wrap_expr(cond: tsast::Expr) -> tsast::Expr {
    tsast::Expr::IfElse {
        cond: Box::new(cond),
        then_expr: Box::new(tsast::Expr::Index {
            object: Box::new(tsast::Expr::Ident("Bool".to_owned())),
            index: Box::new(tsast::Expr::String("true".to_owned())),
        }),
        else_expr: Box::new(tsast::Expr::Index {
            object: Box::new(tsast::Expr::Ident("Bool".to_owned())),
            index: Box::new(tsast::Expr::String("false".to_owned())),
        }),
    }
}

fn specialize_operator_wrapper_expr(
    extern_name: &str,
    params: &[tsast::Param],
) -> Option<tsast::Expr> {
    let operator = parse_extern_operator(extern_name)?;
    match operator {
        ExternOperator::Unary(op) if params.len() == 1 => Some(tsast::Expr::Unary {
            op,
            expr: Box::new(tsast::Expr::Ident(params[0].name.clone())),
        }),
        ExternOperator::Binary(op) if params.len() == 2 => Some(tsast::Expr::Binary {
            left: Box::new(tsast::Expr::Ident(params[0].name.clone())),
            op,
            right: Box::new(tsast::Expr::Ident(params[1].name.clone())),
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternOperator {
    Unary(tsast::UnaryOp),
    Binary(tsast::BinaryOp),
}

fn parse_extern_operator(extern_name: &str) -> Option<ExternOperator> {
    let segment = extern_name.rsplit('.').next().unwrap_or(extern_name);
    if segment.len() >= 3 && segment.starts_with('_') && segment.ends_with('_') {
        return map_binary_operator(&segment[1..segment.len() - 1]).map(ExternOperator::Binary);
    }
    if segment.len() >= 2 && segment.ends_with('_') {
        return map_unary_operator(&segment[..segment.len() - 1]).map(ExternOperator::Unary);
    }
    None
}

fn map_unary_operator(op: &str) -> Option<tsast::UnaryOp> {
    match op {
        "+" => Some(tsast::UnaryOp::Plus),
        "-" => Some(tsast::UnaryOp::Minus),
        "!" => Some(tsast::UnaryOp::Not),
        "~" => Some(tsast::UnaryOp::BitNot),
        _ => None,
    }
}

fn map_binary_operator(op: &str) -> Option<tsast::BinaryOp> {
    match op {
        "+" => Some(tsast::BinaryOp::Add),
        "-" => Some(tsast::BinaryOp::Sub),
        "*" => Some(tsast::BinaryOp::Mul),
        "/" => Some(tsast::BinaryOp::Div),
        "^" | "**" => Some(tsast::BinaryOp::Exp),
        "==" | "===" => Some(tsast::BinaryOp::EqEqEq),
        "!=" | "!==" => Some(tsast::BinaryOp::NotEqEq),
        "<" => Some(tsast::BinaryOp::Lt),
        "<=" => Some(tsast::BinaryOp::Lte),
        ">" => Some(tsast::BinaryOp::Gt),
        ">=" => Some(tsast::BinaryOp::Gte),
        "&&" => Some(tsast::BinaryOp::AndAnd),
        "||" => Some(tsast::BinaryOp::OrOr),
        _ => None,
    }
}

fn lower_data_bundle_const(data: &lir::DataDecl) -> tsast::ConstDecl {
    let init = tsast::Expr::Object(
        data.variants
            .iter()
            .map(|variant| tsast::ObjectProp {
                key: tsast::ObjectKey::String(variant.name.clone()),
                value: lower_variant_ctor_expr(data, variant),
            })
            .collect(),
    );
    tsast::ConstDecl {
        export: true,
        name: data.name.clone(),
        type_ann: Some(tsast::TsType::Raw(render_data_bundle_type(data))),
        init,
    }
}

fn lower_variant_ctor_expr(_data: &lir::DataDecl, variant: &lir::VariantDecl) -> tsast::Expr {
    // `#[as__raw(true|false)]` variants emit the raw JS literal directly.
    // v1 scope is nullary-only (raw variants have no payload).
    if let Some(raw) = &variant.as_raw {
        return raw_value_to_ts_expr(raw);
    }

    if variant.payload.is_empty() {
        let fields = vec![tsast::ObjectProp {
            key: tsast::ObjectKey::Computed(Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned()))),
            value: tsast::Expr::String(variant.name.clone()),
        }];
        return tsast::Expr::Object(fields);
    }

    let params = variant
        .payload
        .iter()
        .enumerate()
        .map(|(index, _)| tsast::Param::new(format!("arg{index}")))
        .collect::<Vec<_>>();
    let mut fields = vec![tsast::ObjectProp {
        key: tsast::ObjectKey::Computed(Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned()))),
        value: tsast::Expr::String(variant.name.clone()),
    }];
    fields.push(tsast::ObjectProp {
        key: tsast::ObjectKey::Ident("args".to_owned()),
        value: tsast::Expr::Array(
            params
                .iter()
                .map(|param| tsast::Expr::Ident(param.name.clone()))
                .collect(),
        ),
    });
    tsast::Expr::Arrow {
        params,
        return_type: None,
        body: Box::new(tsast::FunctionBody::Expr(Box::new(tsast::Expr::Object(
            fields,
        )))),
    }
}

fn render_data_bundle_type(data: &lir::DataDecl) -> String {
    let generics_decl = if data.generics.is_empty() {
        String::new()
    } else {
        format!("<{}>", data.generics.join(", "))
    };
    let result_type = format!("{}{}", data.name, generics_decl);
    let members = data
        .variants
        .iter()
        .map(|variant| {
            let params = variant
                .payload
                .iter()
                .enumerate()
                .map(|(index, payload)| {
                    let payload = type_expr_to_ts_text(&payload.value);
                    format!("arg{index}: {payload}")
                })
                .collect::<Vec<_>>()
                .join(", ");
            if let Some(raw) = &variant.as_raw {
                let lit = match raw {
                    AsRawValue::True => "true",
                    AsRawValue::False => "false",
                };
                return format!("\"{}\": {lit}", variant.name);
            }
            if variant.payload.is_empty() {
                if data.generics.is_empty() {
                    format!("\"{}\": {result_type}", variant.name)
                } else {
                    let never_args = vec!["never"; data.generics.len()].join(", ");
                    format!("\"{}\": {}<{never_args}>", variant.name, data.name)
                }
            } else if data.generics.is_empty() {
                format!("\"{}\": ({params}) => {result_type}", variant.name)
            } else {
                format!(
                    "\"{}\": {}({params}) => {result_type}",
                    variant.name, generics_decl
                )
            }
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("{{ {members} }}")
}

fn type_expr_to_ts_text(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Named(name) if name == "Unit" => "void".to_owned(),
        TypeExpr::Named(name) if name == "<missing>" => "__CpsValue".to_owned(),
        TypeExpr::Named(name) => name.clone(),
        TypeExpr::App { head, args } => format!(
            "{head}<{}>",
            args.iter()
                .map(type_expr_to_ts_text)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        TypeExpr::Produce(inner) => type_expr_to_ts_text(inner),
        TypeExpr::Thunk(inner) => type_expr_to_ts_text(inner),
        TypeExpr::Cap { name, .. } => name.clone(),
    }
}

fn validate_program_has_no_any_or_unknown(program: &tsast::Program) -> Result<(), BackendError> {
    for stmt in &program.body {
        validate_stmt_has_no_any_or_unknown(stmt)?;
    }
    Ok(())
}

fn validate_stmt_has_no_any_or_unknown(stmt: &tsast::Stmt) -> Result<(), BackendError> {
    match stmt {
        tsast::Stmt::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
        tsast::Stmt::Return(expr) => {
            if let Some(expr) = expr {
                validate_expr_has_no_any_or_unknown(expr)?;
            }
            Ok(())
        }
        tsast::Stmt::Const(decl) => {
            if let Some(ty) = &decl.type_ann {
                validate_ts_type_has_no_any_or_unknown(ty)?;
            }
            validate_expr_has_no_any_or_unknown(&decl.init)
        }
        tsast::Stmt::If {
            cond,
            then_branch,
            else_branch,
        } => {
            validate_expr_has_no_any_or_unknown(cond)?;
            for stmt in &then_branch.stmts {
                validate_stmt_has_no_any_or_unknown(stmt)?;
            }
            if let Some(else_branch) = else_branch {
                for stmt in &else_branch.stmts {
                    validate_stmt_has_no_any_or_unknown(stmt)?;
                }
            }
            Ok(())
        }
        tsast::Stmt::Block(block) => {
            for stmt in &block.stmts {
                validate_stmt_has_no_any_or_unknown(stmt)?;
            }
            Ok(())
        }
        tsast::Stmt::Function(decl) => {
            for param in &decl.params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                }
            }
            let Some(ret) = decl.return_type.as_ref() else {
                return Err(BackendError::EmitFailed(format!(
                    "function `{}` has no return type",
                    decl.name
                )));
            };
            validate_ts_type_has_no_any_or_unknown(ret)?;
            match &decl.body {
                tsast::FunctionBody::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
                tsast::FunctionBody::Block(block) => {
                    for stmt in &block.stmts {
                        validate_stmt_has_no_any_or_unknown(stmt)?;
                    }
                    Ok(())
                }
            }
        }
        tsast::Stmt::Let { init, type_ann, .. } => {
            if let Some(ty) = type_ann {
                validate_ts_type_has_no_any_or_unknown(ty)?;
            }
            if let Some(init) = init {
                validate_expr_has_no_any_or_unknown(init)?;
            }
            Ok(())
        }
        tsast::Stmt::Assign { value, .. } => validate_expr_has_no_any_or_unknown(value),
        tsast::Stmt::TypeAlias(alias) => validate_ts_type_has_no_any_or_unknown(&alias.ty),
        tsast::Stmt::Interface(interface) => {
            for member in &interface.members {
                validate_ts_type_has_no_any_or_unknown(&member.ty)?;
            }
            Ok(())
        }
    }
}

fn validate_expr_has_no_any_or_unknown(expr: &tsast::Expr) -> Result<(), BackendError> {
    match expr {
        tsast::Expr::Arrow {
            params,
            return_type,
            body,
        } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                }
            }
            if let Some(ret) = return_type {
                validate_ts_type_has_no_any_or_unknown(ret)?;
            }
            match body.as_ref() {
                tsast::FunctionBody::Expr(expr) => validate_expr_has_no_any_or_unknown(expr),
                tsast::FunctionBody::Block(block) => {
                    for stmt in &block.stmts {
                        validate_stmt_has_no_any_or_unknown(stmt)?;
                    }
                    Ok(())
                }
            }
        }
        tsast::Expr::Unary { expr, .. } => validate_expr_has_no_any_or_unknown(expr),
        tsast::Expr::Binary { left, right, .. } => {
            validate_expr_has_no_any_or_unknown(left)?;
            validate_expr_has_no_any_or_unknown(right)
        }
        tsast::Expr::Void(expr) => validate_expr_has_no_any_or_unknown(expr),
        tsast::Expr::Call { callee, args } => {
            validate_expr_has_no_any_or_unknown(callee)?;
            for arg in args {
                validate_expr_has_no_any_or_unknown(arg)?;
            }
            Ok(())
        }
        tsast::Expr::Member { object, .. } => validate_expr_has_no_any_or_unknown(object),
        tsast::Expr::Index { object, index } => {
            validate_expr_has_no_any_or_unknown(object)?;
            validate_expr_has_no_any_or_unknown(index)
        }
        tsast::Expr::Array(items) => {
            for item in items {
                validate_expr_has_no_any_or_unknown(item)?;
            }
            Ok(())
        }
        tsast::Expr::Object(props) => {
            for prop in props {
                if let tsast::ObjectKey::Computed(expr) = &prop.key {
                    validate_expr_has_no_any_or_unknown(expr)?;
                }
                validate_expr_has_no_any_or_unknown(&prop.value)?;
            }
            Ok(())
        }
        tsast::Expr::IfElse {
            cond,
            then_expr,
            else_expr,
        } => {
            validate_expr_has_no_any_or_unknown(cond)?;
            validate_expr_has_no_any_or_unknown(then_expr)?;
            validate_expr_has_no_any_or_unknown(else_expr)
        }
        tsast::Expr::Ident(_)
        | tsast::Expr::String(_)
        | tsast::Expr::Number(_)
        | tsast::Expr::Bool(_)
        | tsast::Expr::Null
        | tsast::Expr::Undefined => Ok(()),
    }
}

fn validate_ts_type_has_no_any_or_unknown(ty: &tsast::TsType) -> Result<(), BackendError> {
    match ty {
        tsast::TsType::Any => Err(BackendError::EmitFailed(
            "refusing to emit TypeScript `any`".to_owned(),
        )),
        tsast::TsType::Unknown => Err(BackendError::EmitFailed(
            "refusing to emit TypeScript `unknown`".to_owned(),
        )),
        tsast::TsType::Array(inner) => validate_ts_type_has_no_any_or_unknown(inner),
        tsast::TsType::Union(items) => {
            for item in items {
                validate_ts_type_has_no_any_or_unknown(item)?;
            }
            Ok(())
        }
        tsast::TsType::Func { params, ret } => {
            for param in params {
                if let Some(ty) = &param.type_ann {
                    validate_ts_type_has_no_any_or_unknown(ty)?;
                } else {
                    return Err(BackendError::EmitFailed(format!(
                        "function type parameter `{}` has no type",
                        param.name
                    )));
                }
            }
            validate_ts_type_has_no_any_or_unknown(ret)
        }
        tsast::TsType::Raw(_) => Ok(()),
        tsast::TsType::Never
        | tsast::TsType::Void
        | tsast::TsType::Boolean
        | tsast::TsType::Number
        | tsast::TsType::String
        | tsast::TsType::Null
        | tsast::TsType::Undefined
        | tsast::TsType::TypeRef(_) => Ok(()),
    }
}

/// Build an immediately-invoked function expression: ((name) => body)(arg)
fn iife(name: &str, body: tsast::Expr, arg: tsast::Expr) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Arrow {
            params: vec![tsast::Param::new(name)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
        }),
        args: vec![arg],
    }
}

fn lower_expr(expr: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    match expr {
        lir::Expr::Ident { name, .. } if name == "Unit" => {
            tsast::Expr::Void(Box::new(tsast::Expr::Number(0.0)))
        }
        lir::Expr::Ident { name, .. } => tsast::Expr::Ident(name.clone()),
        lir::Expr::String { value, .. } => tsast::Expr::String(value.clone()),
        lir::Expr::Number { value, .. } => {
            tsast::Expr::Number(value.parse::<f64>().unwrap_or(0.0))
        }
        lir::Expr::Produce { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Thunk { expr, .. } => tsast::Expr::Arrow {
            params: Vec::new(),
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(expr, ctx)))),
        },
        lir::Expr::Force { expr, .. } => lower_force_expr(expr, ctx),
        lir::Expr::Lambda { param, body, .. } => tsast::Expr::Arrow {
            params: vec![tsast::Param::new(param)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(lower_expr(body, ctx)))),
        },
        lir::Expr::Apply { callee, arg, .. } => lower_apply_expr(callee, arg, ctx),
        lir::Expr::Unroll { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Roll { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Let {
            name, value, body, ..
        } => iife(name, lower_expr(body, ctx), lower_expr(value, ctx)),
        lir::Expr::Match {
            scrutinee, arms, ..
        } => lower_match_expr(scrutinee, arms, ctx),
        lir::Expr::Ctor { name, args, .. } => {
            if let Some((owner, variant)) = name.split_once('.') {
                // `#[as__raw]` variants: skip the bundle access and emit the
                // raw JS literal directly. v1 scope is nullary-only.
                if let Some(raw) = ctx
                    .ctor_as_raw
                    .get(&(owner.to_owned(), variant.to_owned()))
                {
                    return raw_value_to_ts_expr(raw);
                }
                let ctor = tsast::Expr::Index {
                    object: Box::new(tsast::Expr::Ident(owner.to_owned())),
                    index: Box::new(tsast::Expr::String(variant.to_owned())),
                };
                if args.is_empty() {
                    ctor
                } else {
                    tsast::Expr::Call {
                        callee: Box::new(ctor),
                        args: args.iter().map(|arg| lower_expr(arg, ctx)).collect(),
                    }
                }
            } else {
                let callee = tsast::Expr::Ident(name.clone());
                if args.is_empty() {
                    callee
                } else {
                    tsast::Expr::Call {
                        callee: Box::new(callee),
                        args: args.iter().map(|arg| lower_expr(arg, ctx)).collect(),
                    }
                }
            }
        }
        lir::Expr::Perform { cap, type_args, .. } => {
            cap_bundle_access_expr(cap, type_args)
        }
        lir::Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            // All handles use CPS (deep CPS: effectful functions need continuation threading)
            lower_cps_handle(cap, type_args, handler, body, ctx)
        }
        lir::Expr::Ann { expr, .. } => lower_expr(expr, ctx),
        lir::Expr::Error { .. } => runtime_call("__lumo_error", Vec::new()),
        lir::Expr::Bundle { .. } => {
            // All bundles are cap handlers — use CPS entries (with __k).
            // A bare Bundle outside a `handle` has no specific handled cap; the
            // inner-body CPS lowering treats handled_caps as empty.
            lower_handler_with_resume(expr, &[], ctx)
        }
        lir::Expr::Member {
            object, field, ..
        } => tsast::Expr::Member {
            object: Box::new(lower_expr(object, ctx)),
            property: field.clone(),
        },
    }
}

fn lower_force_expr(expr: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    if let lir::Expr::Ident { name, .. } = expr {
        if let Some(arity) = ctx.direct_callable_arities.get(name) {
            if *arity == 0 {
                return tsast::Expr::Call {
                    callee: Box::new(tsast::Expr::Ident(name.clone())),
                    args: Vec::new(),
                };
            }
            return tsast::Expr::Ident(name.clone());
        }
    }

    tsast::Expr::Call {
        callee: Box::new(lower_expr(expr, ctx)),
        args: Vec::new(),
    }
}

fn lower_apply_expr(callee: &lir::Expr, arg: &lir::Expr, ctx: &LoweringContext) -> tsast::Expr {
    if let Some((root_name, args)) = collect_direct_apply_chain(callee, arg, ctx) {
        return tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Ident(root_name)),
            args: args.into_iter().map(|arg| lower_expr(arg, ctx)).collect(),
        };
    }

    // Try uncurrying member-based call chains: Apply*(Member(Ident(obj), method), args)
    if let Some((obj_name, method, args)) = collect_member_apply_chain(callee, arg, ctx) {
        return tsast::Expr::Call {
            callee: Box::new(tsast::Expr::Member {
                object: Box::new(tsast::Expr::Ident(obj_name)),
                property: method,
            }),
            args: args.into_iter().map(|arg| lower_expr(arg, ctx)).collect(),
        };
    }

    tsast::Expr::Call {
        callee: Box::new(lower_expr(callee, ctx)),
        args: vec![lower_expr(arg, ctx)],
    }
}

fn collect_direct_apply_chain<'a>(
    callee: &'a lir::Expr,
    arg: &'a lir::Expr,
    ctx: &LoweringContext,
) -> Option<(String, Vec<&'a lir::Expr>)> {
    let mut args = vec![arg];
    let mut cursor = callee;

    while let lir::Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = cursor
    {
        args.push(inner_arg.as_ref());
        cursor = inner_callee.as_ref();
    }

    let lir::Expr::Force { expr, .. } = cursor else {
        return None;
    };
    let lir::Expr::Ident { name, .. } = expr.as_ref() else {
        return None;
    };
    if ctx.direct_callable_arities.get(name).copied().unwrap_or(0) == 0 {
        return None;
    }

    args.reverse();
    Some((name.clone(), args))
}

/// Uncurry member-based call chains: `Apply*(Member(Ident(obj), method), args)`
/// into `(obj_name, method, args)` when the impl method arity matches.
fn collect_member_apply_chain<'a>(
    callee: &'a lir::Expr,
    arg: &'a lir::Expr,
    ctx: &LoweringContext,
) -> Option<(String, String, Vec<&'a lir::Expr>)> {
    let mut args = vec![arg];
    let mut cursor = callee;

    while let lir::Expr::Apply {
        callee: inner_callee,
        arg: inner_arg,
        ..
    } = cursor
    {
        args.push(inner_arg.as_ref());
        cursor = inner_callee.as_ref();
    }

    // LIR member calls use Member directly (no Force wrapper)
    let lir::Expr::Member { object, field, .. } = cursor else {
        return None;
    };
    let lir::Expr::Ident { name: obj_name, .. } = object.as_ref() else {
        return None;
    };
    let key = (obj_name.clone(), field.clone());
    if !ctx.impl_method_arities.contains_key(&key) {
        return None;
    }

    args.reverse();
    Some((obj_name.clone(), field.clone(), args))
}

fn lower_match_expr(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let lowered_scrutinee = lower_expr(scrutinee, ctx);
    let scrutinee_name = ctx.next_match_name();
    let scrutinee_expr = tsast::Expr::Ident(scrutinee_name.clone());
    let rows = arms
        .iter()
        .map(|arm| MatchRow {
            patterns: vec![pattern_to_match_pattern(&arm.pattern)],
            bindings: Vec::new(),
            body: arm.body.clone(),
        })
        .collect::<Vec<_>>();
    let decision = build_match_decision(vec![scrutinee_expr.clone()], rows);
    let lowered =
        lower_match_decision(&scrutinee_expr, decision, &ctx.variant_as_raw, &|body, bindings| {
            wrap_bindings(lower_expr(body, ctx), bindings)
        });

    iife(&scrutinee_name, lowered, lowered_scrutinee)
}


/// Wrap an expression with IIFE bindings: ((name) => expr)(value) for each binding.
fn wrap_bindings(mut expr: tsast::Expr, bindings: Vec<(String, tsast::Expr)>) -> tsast::Expr {
    for (name, value) in bindings.into_iter().rev() {
        expr = iife(&name, expr, value);
    }
    expr
}

fn payload_access_expr(value: &tsast::Expr, index: usize) -> tsast::Expr {
    tsast::Expr::Index {
        object: Box::new(tsast::Expr::Member {
            object: Box::new(value.clone()),
            property: "args".to_owned(),
        }),
        index: Box::new(tsast::Expr::Number(index as f64)),
    }
}

fn is_irrefutable_pattern(pattern: &MatchPattern) -> bool {
    matches!(pattern, MatchPattern::Wildcard | MatchPattern::Bind(_))
}

#[derive(Debug, Clone)]
struct MatchRow {
    patterns: Vec<MatchPattern>,
    bindings: Vec<(String, tsast::Expr)>,
    body: lir::Expr,
}

#[derive(Debug, Clone)]
enum MatchDecision {
    Fail,
    Leaf {
        bindings: Vec<(String, tsast::Expr)>,
        body: lir::Expr,
    },
    Switch {
        occurrence: tsast::Expr,
        cases: Vec<MatchCase>,
        default: Box<MatchDecision>,
    },
}

#[derive(Debug, Clone)]
struct MatchCase {
    ctor_name: String,
    subtree: MatchDecision,
}

fn build_match_decision(occurrences: Vec<tsast::Expr>, rows: Vec<MatchRow>) -> MatchDecision {
    if rows.is_empty() {
        return MatchDecision::Fail;
    }
    if rows[0].patterns.iter().all(is_irrefutable_pattern) {
        let mut bindings = rows[0].bindings.clone();
        for (pattern, occurrence) in rows[0].patterns.iter().zip(occurrences.iter()) {
            collect_irrefutable_bindings(pattern, occurrence, &mut bindings);
        }
        return MatchDecision::Leaf {
            bindings,
            body: rows[0].body.clone(),
        };
    }

    let Some(column) = rows[0]
        .patterns
        .iter()
        .position(|pattern| !is_irrefutable_pattern(pattern))
    else {
        return MatchDecision::Leaf {
            bindings: rows[0].bindings.clone(),
            body: rows[0].body.clone(),
        };
    };

    let occurrence = occurrences[column].clone();
    let default_rows = default_specialize_rows(&rows, column, &occurrence);
    let default_occurrences = default_specialize_occurrences(&occurrences, column);
    let default = if default_rows.is_empty() {
        MatchDecision::Fail
    } else {
        build_match_decision(default_occurrences, default_rows)
    };

    let cases = collect_ctor_cases(&rows, column)
        .into_iter()
        .map(|(ctor_name, arity)| MatchCase {
            ctor_name: ctor_name.clone(),
            subtree: build_match_decision(
                specialize_occurrences_for_ctor(&occurrences, column, arity),
                specialize_rows_for_ctor(&rows, column, &ctor_name, arity, &occurrence),
            ),
        })
        .collect::<Vec<_>>();

    MatchDecision::Switch {
        occurrence,
        cases,
        default: Box::new(default),
    }
}

fn collect_irrefutable_bindings(
    pattern: &MatchPattern,
    occurrence: &tsast::Expr,
    bindings: &mut Vec<(String, tsast::Expr)>,
) {
    if let MatchPattern::Bind(name) = pattern {
        bindings.push((name.clone(), occurrence.clone()));
    }
}

fn collect_ctor_cases(rows: &[MatchRow], column: usize) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    for row in rows {
        let Some(pattern) = row.patterns.get(column) else {
            continue;
        };
        let MatchPattern::Ctor { name, args } = pattern else {
            continue;
        };
        if out.iter().all(|(seen, _)| seen != name) {
            out.push((name.clone(), args.len()));
        }
    }
    out
}

fn specialize_rows_for_ctor(
    rows: &[MatchRow],
    column: usize,
    ctor_name: &str,
    arity: usize,
    occurrence: &tsast::Expr,
) -> Vec<MatchRow> {
    rows.iter()
        .filter_map(|row| specialize_row_for_ctor(row, column, ctor_name, arity, occurrence))
        .collect()
}

fn specialize_row_for_ctor(
    row: &MatchRow,
    column: usize,
    ctor_name: &str,
    arity: usize,
    occurrence: &tsast::Expr,
) -> Option<MatchRow> {
    let pattern = row.patterns.get(column)?;
    let mut patterns = row.patterns[..column].to_vec();
    let mut bindings = row.bindings.clone();
    match pattern {
        MatchPattern::Ctor { name, args } if name == ctor_name && args.len() == arity => {
            patterns.extend(args.clone());
        }
        MatchPattern::Wildcard => {
            patterns.extend(std::iter::repeat_n(MatchPattern::Wildcard, arity));
        }
        MatchPattern::Bind(name) => {
            bindings.push((name.clone(), occurrence.clone()));
            patterns.extend(std::iter::repeat_n(MatchPattern::Wildcard, arity));
        }
        _ => return None,
    }
    patterns.extend_from_slice(&row.patterns[column + 1..]);
    Some(MatchRow {
        patterns,
        bindings,
        body: row.body.clone(),
    })
}

fn default_specialize_rows(
    rows: &[MatchRow],
    column: usize,
    occurrence: &tsast::Expr,
) -> Vec<MatchRow> {
    rows.iter()
        .filter_map(|row| {
            let pattern = row.patterns.get(column)?;
            let mut patterns = row.patterns[..column].to_vec();
            let mut bindings = row.bindings.clone();
            match pattern {
                MatchPattern::Wildcard => {}
                MatchPattern::Bind(name) => bindings.push((name.clone(), occurrence.clone())),
                MatchPattern::Ctor { .. } => return None,
            }
            patterns.extend_from_slice(&row.patterns[column + 1..]);
            Some(MatchRow {
                patterns,
                bindings,
                body: row.body.clone(),
            })
        })
        .collect()
}

fn specialize_occurrences_for_ctor(
    occurrences: &[tsast::Expr],
    column: usize,
    arity: usize,
) -> Vec<tsast::Expr> {
    let mut out = occurrences[..column].to_vec();
    let occurrence = &occurrences[column];
    out.extend((0..arity).map(|index| payload_access_expr(occurrence, index)));
    out.extend_from_slice(&occurrences[column + 1..]);
    out
}

fn default_specialize_occurrences(occurrences: &[tsast::Expr], column: usize) -> Vec<tsast::Expr> {
    let mut out = occurrences[..column].to_vec();
    out.extend_from_slice(&occurrences[column + 1..]);
    out
}

fn lower_match_decision(
    error_value: &tsast::Expr,
    decision: MatchDecision,
    variant_as_raw: &HashMap<String, AsRawValue>,
    lower_body: &dyn Fn(&lir::Expr, Vec<(String, tsast::Expr)>) -> tsast::Expr,
) -> tsast::Expr {
    match decision {
        MatchDecision::Fail => runtime_call("__lumo_match_error", vec![error_value.clone()]),
        MatchDecision::Leaf { bindings, body } => lower_body(&body, bindings),
        MatchDecision::Switch {
            occurrence,
            mut cases,
            default,
        } => {
            // When default is Fail (exhaustive match), use the last case as the unconditional
            // else branch — TypeScript's type system guarantees no other value is possible.
            let mut folded = if matches!(*default, MatchDecision::Fail) && !cases.is_empty() {
                let last = cases.pop().unwrap();
                lower_match_decision(error_value, last.subtree, variant_as_raw, lower_body)
            } else {
                lower_match_decision(error_value, *default, variant_as_raw, lower_body)
            };
            for case in cases.into_iter().rev() {
                // `#[as__raw]` variant: compare with the raw JS literal
                // (`=== true` / `=== false`) instead of the tagged form.
                let cond = if let Some(raw) = variant_as_raw.get(&case.ctor_name) {
                    tsast::Expr::Binary {
                        left: Box::new(occurrence.clone()),
                        op: tsast::BinaryOp::EqEqEq,
                        right: Box::new(raw_value_to_ts_expr(raw)),
                    }
                } else {
                    // Inline `__lumo_is(occurrence, "ctor")` → `occurrence[LUMO_TAG] === "ctor"`.
                    // Match scrutinees are guaranteed ADT values by typecheck, so no null-guard needed.
                    tsast::Expr::Binary {
                        left: Box::new(tsast::Expr::Index {
                            object: Box::new(occurrence.clone()),
                            index: Box::new(tsast::Expr::Ident("LUMO_TAG".to_owned())),
                        }),
                        op: tsast::BinaryOp::EqEqEq,
                        right: Box::new(tsast::Expr::String(case.ctor_name)),
                    }
                };
                folded = tsast::Expr::IfElse {
                    cond: Box::new(cond),
                    then_expr: Box::new(lower_match_decision(
                        error_value,
                        case.subtree,
                        variant_as_raw,
                        lower_body,
                    )),
                    else_expr: Box::new(folded),
                };
            }
            folded
        }
    }
}

// --- CPS codegen for cap handlers with resume ---

/// Decompose a perform call pattern in LIR:
/// Apply*(Member(Perform(E), op), args) → Some((E, op, args))
/// Member(Perform(E), op) with no args → Some((E, op, []))
/// Peel Apply nodes off an expression, returning (root, args) in application order.
fn unwrap_apply_chain<'a>(expr: &'a lir::Expr) -> (&'a lir::Expr, Vec<&'a lir::Expr>) {
    let mut args = Vec::new();
    let mut cursor = expr;
    while let lir::Expr::Apply { callee, arg, .. } = cursor {
        args.push(arg.as_ref());
        cursor = callee.as_ref();
    }
    args.reverse();
    (cursor, args)
}

fn decompose_perform_call<'a>(
    expr: &'a lir::Expr,
) -> Option<(String, &'a str, Vec<&'a lir::Expr>)> {
    let (root, args) = unwrap_apply_chain(expr);
    // Unwrap Force wrapper: LIR produces Apply*(Force(Member(Perform(cap), op)), args)
    let root = if let lir::Expr::Force { expr, .. } = root {
        expr.as_ref()
    } else {
        root
    };
    if let lir::Expr::Member { object, field, .. } = root {
        if let lir::Expr::Perform { cap, type_args, .. } = object.as_ref() {
            return Some((cap_runtime_name(cap, type_args), field.as_str(), args));
        }
    }
    None
}

/// Decompose an impl method call pattern: `Apply*(Member(Ident(obj), method), args)`
/// or `Apply*(Force(Member(Ident(obj), method)), args)`.
/// Returns `(obj_const_name, method_name, args)` when the shape matches.
fn decompose_impl_method_call<'a>(
    expr: &'a lir::Expr,
) -> Option<(&'a str, &'a str, Vec<&'a lir::Expr>)> {
    let (root, args) = unwrap_apply_chain(expr);
    let root = if let lir::Expr::Force { expr, .. } = root {
        expr.as_ref()
    } else {
        root
    };
    if let lir::Expr::Member { object, field, .. } = root {
        if let lir::Expr::Ident { name, .. } = object.as_ref() {
            return Some((name.as_str(), field.as_str(), args));
        }
    }
    None
}

/// Decompose a function call pattern in LIR:
/// Apply*(Force(Ident(name)), args) → Some((name, args))
/// Force(Ident(name)) with no args → Some((name, []))
fn decompose_fn_call<'a>(expr: &'a lir::Expr) -> Option<(&'a str, Vec<&'a lir::Expr>)> {
    let (root, args) = unwrap_apply_chain(expr);
    if let lir::Expr::Force { expr, .. } = root {
        if let lir::Expr::Ident { name, .. } = expr.as_ref() {
            return Some((name.as_str(), args));
        }
    }
    None
}

/// Compile handle expression with CPS for resume support.
/// Extends the ambient `__caps` bundle with the new handler under the cap's
/// bundle key, and evaluates the body in that extended scope.
fn lower_cps_handle(
    cap: &str,
    type_args: &[String],
    handler: &lir::Expr,
    body: &lir::Expr,
    ctx: &LoweringContext,
) -> tsast::Expr {
    let runtime_name = cap_runtime_name(cap, type_args);
    let bundle_key = cap_bundle_key(cap, type_args);
    let handled = vec![runtime_name.clone()];
    // At the top-level handle site, the outer continuation is the identity —
    // the whole `handle` expression's value is returned directly. Both the
    // body's CPS continuation and the factory's `__k_handle` are the shared
    // `__identity` const (see runtime prelude).
    let cps_body = lower_cps_expr(body, identity_k_expr(), &handled, ctx);
    let handler_factory = lower_handler_with_resume(handler, &handled, ctx);
    let handler_instance = tsast::Expr::Call {
        callee: Box::new(handler_factory),
        args: vec![identity_k_expr()],
    };
    let extended = extended_caps_object(&bundle_key, handler_instance);
    let bound = iife(CAPS_PARAM, cps_body, extended);
    // Wrap in trampoline to evaluate CPS thunks iteratively
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident("__trampoline".to_owned())),
        args: vec![bound],
    }
}

/// Build `Object.assign({}, __caps, { <key>: <handler> })` — an extended caps
/// bundle that layers a new handler on top of the outer scope's `__caps`.
/// Evaluated in the outer scope (so `__caps` here refers to the enclosing
/// bundle). Using `Object.assign` rather than spread syntax keeps the tsast
/// AST minimal.
fn extended_caps_object(bundle_key: &str, handler: tsast::Expr) -> tsast::Expr {
    let new_entry = tsast::Expr::Object(vec![tsast::ObjectProp {
        key: tsast::ObjectKey::Ident(bundle_key.to_owned()),
        value: handler,
    }]);
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Member {
            object: Box::new(tsast::Expr::Ident("Object".to_owned())),
            property: "assign".to_owned(),
        }),
        args: vec![
            tsast::Expr::Object(Vec::new()),
            tsast::Expr::Ident(CAPS_PARAM.to_owned()),
            new_entry,
        ],
    }
}

/// CPS-transform an expression within a handle body.
/// `k` is the continuation expression (a JS function taking a value).
/// `handled_caps` are the caps available for CPS threading.
/// Recursively check whether a LIR expression contains any effectful computation
/// that requires CPS sequencing within the current handled-cap context.
///
/// **General rule**: In CPS mode, every sub-expression in value position (function
/// arguments, match scrutinees, `produce` bodies) must be checked with this function.
/// If it returns true, the sub-expression must be CPS-lowered and bound to a temp
/// variable before use, rather than being lowered with `lower_expr`.
fn is_effectful_expr(
    expr: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> bool {
    // Decompose as perform call: Apply*(Member(Perform(E), op), args)
    if let Some((cap, _, args)) = decompose_perform_call(expr) {
        if handled_caps.iter().any(|c| c == &cap) {
            return true;
        }
        // Even if this cap isn't handled, args might be effectful
        return args
            .iter()
            .any(|a| is_effectful_expr(a, handled_caps, ctx));
    }
    // Decompose as effectful function call: Apply*(Force(Ident(f)), args)
    if let Some((fn_name, args)) = decompose_fn_call(expr) {
        if ctx
            .fn_caps
            .get(fn_name)
            .map_or(false, |c| !c.is_empty())
        {
            return true;
        }
        return args
            .iter()
            .any(|a| is_effectful_expr(a, handled_caps, ctx));
    }
    // Decompose as effectful impl method call: Apply*(Member(Ident(obj), method), args)
    if let Some((obj, method, args)) = decompose_impl_method_call(expr) {
        if ctx
            .impl_method_caps
            .contains_key(&(obj.to_owned(), method.to_owned()))
        {
            return true;
        }
        return args
            .iter()
            .any(|a| is_effectful_expr(a, handled_caps, ctx));
    }
    // Recurse into compound forms
    match expr {
        lir::Expr::Apply { callee, arg, .. } => {
            is_effectful_expr(callee, handled_caps, ctx)
                || is_effectful_expr(arg, handled_caps, ctx)
        }
        lir::Expr::Force { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
        lir::Expr::Match {
            scrutinee, arms, ..
        } => {
            is_effectful_expr(scrutinee, handled_caps, ctx)
                || arms
                    .iter()
                    .any(|a| is_effectful_expr(&a.body, handled_caps, ctx))
        }
        lir::Expr::Let { value, body, .. } => {
            is_effectful_expr(value, handled_caps, ctx)
                || is_effectful_expr(body, handled_caps, ctx)
        }
        lir::Expr::Produce { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
        lir::Expr::Member { object, .. } => is_effectful_expr(object, handled_caps, ctx),
        lir::Expr::Handle { .. } => true,
        // Leaf values are never effectful
        lir::Expr::Ident { .. }
        | lir::Expr::String { .. }
        | lir::Expr::Number { .. }
        | lir::Expr::Perform { .. } // bare Perform (no member/apply) is just a value ref
        | lir::Expr::Bundle { .. }
        | lir::Expr::Lambda { .. }
        | lir::Expr::Error { .. } => false,
        // Wrappers: recurse into inner expression
        lir::Expr::Ctor { args, .. } => args.iter().any(|a| is_effectful_expr(a, handled_caps, ctx)),
        lir::Expr::Roll { expr, .. }
        | lir::Expr::Unroll { expr, .. }
        | lir::Expr::Thunk { expr, .. }
        | lir::Expr::Ann { expr, .. } => is_effectful_expr(expr, handled_caps, ctx),
    }
}

/// Lower a value-position expression within CPS context.
///
/// If the expression is effectful (contains perform/effectful-call), CPS-sequence
/// it: lower it with `lower_cps_expr` into a fresh temp variable, then build the
/// continuation with that variable.
///
/// If it is pure, lower it directly with `lower_expr` and build immediately.
fn lower_cps_value(
    expr: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
    then: impl FnOnce(tsast::Expr) -> tsast::Expr,
) -> tsast::Expr {
    if is_effectful_expr(expr, handled_caps, ctx) {
        static COUNTER: std::sync::atomic::AtomicUsize =
            std::sync::atomic::AtomicUsize::new(0);
        let tmp = format!(
            "__cps_v_{}",
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let body = then(tsast::Expr::Ident(tmp.clone()));
        let inner_k = tsast::Expr::Arrow {
            params: vec![tsast::Param::new(&tmp)],
            return_type: None,
            body: Box::new(tsast::FunctionBody::Expr(Box::new(body))),
        };
        lower_cps_expr(expr, inner_k, handled_caps, ctx)
    } else {
        then(lower_expr(expr, ctx))
    }
}

/// Lower a list of value-position expressions within CPS context, left-to-right.
/// Each effectful expression is CPS-sequenced before the next one is evaluated.
fn lower_cps_values(
    exprs: &[&lir::Expr],
    idx: usize,
    acc: Vec<tsast::Expr>,
    handled_caps: &[String],
    ctx: &LoweringContext,
    then: &dyn Fn(Vec<tsast::Expr>) -> tsast::Expr,
) -> tsast::Expr {
    if idx >= exprs.len() {
        return then(acc);
    }
    lower_cps_value(exprs[idx], handled_caps, ctx, |val| {
        let mut next_acc = acc;
        next_acc.push(val);
        lower_cps_values(exprs, idx + 1, next_acc, handled_caps, ctx, then)
    })
}

fn lower_cps_expr(
    expr: &lir::Expr,
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    lower_cps_expr_inner(expr, k, handled_caps, ctx)
}

fn lower_cps_expr_inner(
    expr: &lir::Expr,
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    // Check if this is a perform of a handled cap
    if let Some((cap_runtime, op, args)) = decompose_perform_call(expr) {
        if handled_caps.iter().any(|c| c == &cap_runtime) {
            let op = op.to_owned();
            let cap_access = cap_bundle_access_from_runtime(&cap_runtime);
            // CPS-sequence all arguments, then emit the perform call.
            // Spec: `__caps` is the FIRST argument, user args follow, `__k` last.
            // - handler bundle methods accept `(__caps, args..., __k)` and typically ignore __caps
            // - CPS-form impl methods actually consume __caps to dispatch their own performs
            return lower_cps_values(&args, 0, Vec::new(), handled_caps, ctx, &|ts_args| {
                let callee = tsast::Expr::Member {
                    object: Box::new(cap_access.clone()),
                    property: op.clone(),
                };
                let mut final_args = vec![tsast::Expr::Ident(CAPS_PARAM.to_owned())];
                final_args.extend(ts_args);
                final_args.push(k.clone());
                tsast::Expr::Call {
                    callee: Box::new(callee),
                    args: final_args,
                }
            });
        }
    }

    match expr {
        lir::Expr::Produce { expr: inner, .. } => {
            // produce v → CPS-lower v (in case it's effectful), then k(result)
            lower_cps_value(inner, handled_caps, ctx, |val| tsast::Expr::Call {
                callee: Box::new(k),
                args: vec![val],
            })
        }
        lir::Expr::Let {
            name, value, body, ..
        } => {
            // let x = comp in body → CPS[comp]((x) => CPS[body](k))
            let body_cps = lower_cps_expr(body, k, handled_caps, ctx);
            let inner_k = tsast::Expr::Arrow {
                params: vec![tsast::Param::new(name)],
                return_type: None,
                body: Box::new(tsast::FunctionBody::Expr(Box::new(body_cps))),
            };
            lower_cps_expr(value, inner_k, handled_caps, ctx)
        }
        lir::Expr::Match {
            scrutinee, arms, ..
        } => lower_cps_match_expr(scrutinee, arms, k, handled_caps, ctx),
        // Ctor with effectful args: CPS-sequence each arg, then construct
        lir::Expr::Ctor { name, args, .. } => {
            let arg_refs: Vec<&lir::Expr> = args.iter().collect();
            let name = name.clone();
            lower_cps_values(&arg_refs, 0, Vec::new(), handled_caps, ctx, &|ts_args| {
                let ctor_expr = if let Some((owner, variant)) = name.split_once('.') {
                    // `#[as__raw]` variants: emit raw literal (no bundle access).
                    if let Some(raw) = ctx
                        .ctor_as_raw
                        .get(&(owner.to_owned(), variant.to_owned()))
                    {
                        raw_value_to_ts_expr(raw)
                    } else {
                        let callee = tsast::Expr::Index {
                            object: Box::new(tsast::Expr::Ident(owner.to_owned())),
                            index: Box::new(tsast::Expr::String(variant.to_owned())),
                        };
                        if ts_args.is_empty() {
                            callee
                        } else {
                            tsast::Expr::Call {
                                callee: Box::new(callee),
                                args: ts_args,
                            }
                        }
                    }
                } else {
                    let callee = tsast::Expr::Ident(name.clone());
                    if ts_args.is_empty() {
                        callee
                    } else {
                        tsast::Expr::Call {
                            callee: Box::new(callee),
                            args: ts_args,
                        }
                    }
                };
                tsast::Expr::Call {
                    callee: Box::new(k.clone()),
                    args: vec![ctor_expr],
                }
            })
        }
        // Transparent wrappers: recurse into inner expression for CPS
        lir::Expr::Unroll { expr, .. }
        | lir::Expr::Roll { expr, .. }
        | lir::Expr::Ann { expr, .. } => lower_cps_expr(expr, k, handled_caps, ctx),
        lir::Expr::Handle {
            cap, type_args, handler, body, ..
        } => {
            // handle Cap with handler in body → extend the ambient __caps bundle
            // with this cap's handler, then CPS-lower body in the extended scope.
            // Factory receives `k` (the outer `handle` continuation) as
            // __k_handle so the handler body's value aborts into `k`.
            let runtime_name = cap_runtime_name(cap, type_args);
            let bundle_key = cap_bundle_key(cap, type_args);
            let mut extended_caps: Vec<String> =
                handled_caps.iter().cloned().collect();
            extended_caps.push(runtime_name.clone());
            let handler_factory = lower_handler_with_resume(handler, &extended_caps, ctx);
            let handler_instance = tsast::Expr::Call {
                callee: Box::new(handler_factory),
                args: vec![k.clone()],
            };
            let cps_body = lower_cps_expr(body, k, &extended_caps, ctx);
            let extended = extended_caps_object(&bundle_key, handler_instance);
            iife(CAPS_PARAM, cps_body, extended)
        }
        // Check for effectful function calls: Apply*(Force(Ident(f)), args)
        _ => {
            if let Some((fn_name, args)) = decompose_fn_call(expr) {
                // Resume handling: `resume(v)` inside a handler body calls
                // the perform-site continuation. Two emission modes:
                //
                // - Tail (`k` is `__k_handle`, i.e. resume is the body's
                //   final expression): emit as `__k_handle(__k_perform(v))`.
                //   `__k_perform(v)` returns a thunk; the outer trampoline
                //   unwinds it. Stack-safe through any perform depth.
                //
                // - Non-tail (e.g. `let _ = resume(a); rest`): drive
                //   `__k_perform(v)` through a synchronous `__trampoline`
                //   at the call site so the continuation's side effects run
                //   before control returns here and `k` is applied. Enables
                //   multi-shot composition at the cost of O(depth) stack.
                if fn_name == "resume" {
                    let is_tail = matches!(&k, tsast::Expr::Ident(n) if n == "__k_handle");
                    return lower_cps_values(
                        &args,
                        0,
                        Vec::new(),
                        handled_caps,
                        ctx,
                        &|ts_args| {
                            let kperform_call = tsast::Expr::Call {
                                callee: Box::new(tsast::Expr::Ident("__k_perform".to_owned())),
                                args: ts_args,
                            };
                            let resume_value = if is_tail {
                                kperform_call
                            } else {
                                tsast::Expr::Call {
                                    callee: Box::new(tsast::Expr::Ident(
                                        "__trampoline".to_owned(),
                                    )),
                                    args: vec![kperform_call],
                                }
                            };
                            tsast::Expr::Call {
                                callee: Box::new(k.clone()),
                                args: vec![resume_value],
                            }
                        },
                    );
                }
                if let Some(caps) = ctx.fn_caps.get(fn_name) {
                    if !caps.is_empty() {
                        let fn_name = fn_name.to_owned();
                        // CPS-sequence all arguments, then emit the effectful call.
                        // Spec: `__caps` is the FIRST argument, user args follow,
                        // continuation `__k` is last.
                        return lower_cps_values(
                            &args,
                            0,
                            Vec::new(),
                            handled_caps,
                            ctx,
                            &|ts_args| {
                                let mut final_args =
                                    vec![tsast::Expr::Ident(CAPS_PARAM.to_owned())];
                                final_args.extend(ts_args);
                                final_args.push(k.clone());
                                tsast::Expr::Call {
                                    callee: Box::new(tsast::Expr::Ident(fn_name.clone())),
                                    args: final_args,
                                }
                            },
                        );
                    }
                }
            }
            // Effectful impl method call: Apply*(Member(Ident(obj), method), args)
            if let Some((obj, method, args)) = decompose_impl_method_call(expr) {
                if ctx
                    .impl_method_caps
                    .contains_key(&(obj.to_owned(), method.to_owned()))
                {
                    let obj = obj.to_owned();
                    let method = method.to_owned();
                    return lower_cps_values(
                        &args,
                        0,
                        Vec::new(),
                        handled_caps,
                        ctx,
                        &|ts_args| {
                            let mut final_args =
                                vec![tsast::Expr::Ident(CAPS_PARAM.to_owned())];
                            final_args.extend(ts_args);
                            final_args.push(k.clone());
                            tsast::Expr::Call {
                                callee: Box::new(tsast::Expr::Member {
                                    object: Box::new(tsast::Expr::Ident(obj.clone())),
                                    property: method.clone(),
                                }),
                                args: final_args,
                            }
                        },
                    );
                }
            }
            // Pure fn call with effectful args: Apply*(Force(Ident(f)), args).
            // The fn itself is pure (no caps), but an argument contains a
            // Perform or effectful call — we must CPS-sequence each arg before
            // invoking the pure fn, then pass the result to `k`.
            if let Some((fn_name, args)) = decompose_fn_call(expr) {
                let args_effectful = args
                    .iter()
                    .any(|a| is_effectful_expr(a, handled_caps, ctx));
                if args_effectful {
                    let fn_name = fn_name.to_owned();
                    return lower_cps_values(
                        &args,
                        0,
                        Vec::new(),
                        handled_caps,
                        ctx,
                        &|ts_args| tsast::Expr::Call {
                            callee: Box::new(k.clone()),
                            args: vec![tsast::Expr::Call {
                                callee: Box::new(tsast::Expr::Ident(fn_name.clone())),
                                args: ts_args,
                            }],
                        },
                    );
                }
            }
            // Opaque expression: lower directly and pass result to k.
            // All effectful expression forms (Perform, Handle, Match with effects, etc.)
            // must be handled in explicit branches above — the fallthrough is for pure values.
            tsast::Expr::Call {
                callee: Box::new(k),
                args: vec![lower_expr(expr, ctx)],
            }
        }
    }
}

/// CPS-transform a match expression.
/// The scrutinee is CPS-lowered via `lower_cps_value` so any effectful
/// sub-expression is correctly sequenced before the match body runs.
fn lower_cps_match_expr(
    scrutinee: &lir::Expr,
    arms: &[lir::MatchArm],
    k: tsast::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let k_name = ctx.next_k_name();
    let k_ident = tsast::Expr::Ident(k_name.clone());

    let scrutinee_name = ctx.next_match_name();
    let scrutinee_expr = tsast::Expr::Ident(scrutinee_name.clone());

    let rows = arms
        .iter()
        .map(|arm| MatchRow {
            patterns: vec![pattern_to_match_pattern(&arm.pattern)],
            bindings: Vec::new(),
            body: arm.body.clone(),
        })
        .collect::<Vec<_>>();
    let decision = build_match_decision(vec![scrutinee_expr.clone()], rows);
    let lowered =
        lower_match_decision(&scrutinee_expr, decision, &ctx.variant_as_raw, &|body, bindings| {
            wrap_bindings(
                lower_cps_expr(body, k_ident.clone(), handled_caps, ctx),
                bindings,
            )
        });

    lower_cps_value(scrutinee, handled_caps, ctx, |lowered_scrutinee| {
        let inner = iife(&scrutinee_name, lowered, lowered_scrutinee);
        iife(&k_name, inner, k.clone())
    })
}

/// Emit a single handler method property:
/// `op: (__caps, user_args..., __k_perform) => __thunk(() => body)` where
/// `body` is CPS-lowered with `__k_handle` as the continuation and, if the
/// source references `resume`, wrapped in `((resume) => body)(() => (v) =>
/// __trampoline(__k_perform(v)))`.
///
/// Shared between `lower_handler_with_resume` (for user `handle` blocks)
/// and `lower_impl_const` (for `impl Cap` / `impl T: Cap` default bundles
/// that get installed at main entry as implicit handles).
fn emit_handler_method_prop(
    name: &str,
    user_params: &[lir::Param],
    body: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::ObjectProp {
    let k_handle = tsast::Expr::Ident("__k_handle".to_owned());
    let body_lowered = lower_cps_expr(body, k_handle, handled_caps, ctx);

    let mut params: Vec<tsast::Param> =
        vec![tsast::Param::new(CAPS_PARAM).with_type(caps_type())];
    params.extend(user_params.iter().map(|p| {
        tsast::Param::new(&p.name).with_type(lower_type_expr_to_ts_type(&p.ty.value))
    }));
    params.push(tsast::Param::new("__k_perform").with_type(kont_type_any()));

    // `resume(v)` is emitted directly by the CPS lowering — it doesn't use
    // an IIFE-bound local `resume`. Each call site is classified by its
    // continuation:
    //
    // - Tail (`resume(v)` is the body's last expression, CPS k = __k_handle):
    //   emit as `__k_handle(__k_perform(v))`. The thunk returned by
    //   `__k_perform(v)` bubbles up to the outer `__trampoline` for
    //   iterative unwinding — stack-safe through any perform depth.
    //
    // - Non-tail (`let _ = resume(a); rest` or any composition): emit as
    //   `__trampoline(__k_perform(v))` at the call site, driving the
    //   continuation to a concrete value synchronously so side effects run.
    //   Multi-shot works at the cost of O(call-depth) stack frames, which
    //   is acceptable because non-tail resume is rare outside user-written
    //   non-deterministic handlers.
    //
    // See `lower_cps_expr_inner` for the detection + emission logic.
    let raw_body = body_lowered;

    let inner_body = thunk_wrap(raw_body);
    tsast::ObjectProp {
        key: tsast::ObjectKey::Ident(name.to_owned()),
        value: tsast::Expr::Arrow {
            params,
            return_type: Some(ret_type()),
            body: Box::new(tsast::FunctionBody::Expr(Box::new(inner_body))),
        },
    }
}

/// Wrap a list of handler method properties in the `(__k_handle) => { ... }`
/// factory. Calling `factory(__k_handle)` installs the bundle into a
/// specific handle-expression's context.
fn emit_handler_factory(methods: Vec<tsast::ObjectProp>) -> tsast::Expr {
    tsast::Expr::Arrow {
        params: vec![tsast::Param::new("__k_handle").with_type(kont_type_any())],
        return_type: None,
        body: Box::new(tsast::FunctionBody::Expr(Box::new(tsast::Expr::Object(
            methods,
        )))),
    }
}

/// Build a handler factory for `handle Cap with bundle { ... } in body`.
/// Delegates per-method emission to `emit_handler_method_prop`.
fn lower_handler_with_resume(
    handler: &lir::Expr,
    handled_caps: &[String],
    ctx: &LoweringContext,
) -> tsast::Expr {
    let lir::Expr::Bundle { entries, .. } = handler else {
        return lower_expr(handler, ctx);
    };
    let props: Vec<tsast::ObjectProp> = entries
        .iter()
        .map(|entry| {
            emit_handler_method_prop(
                &entry.name,
                &entry.params,
                &entry.body,
                handled_caps,
                ctx,
            )
        })
        .collect();
    emit_handler_factory(props)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MatchPattern {
    Wildcard,
    Bind(String),
    Ctor {
        name: String,
        args: Vec<MatchPattern>,
    },
}

/// Convert from shared `Pattern` type to backend-local `MatchPattern`.
fn pattern_to_match_pattern(pattern: &Pattern) -> MatchPattern {
    match pattern {
        Pattern::Wildcard => MatchPattern::Wildcard,
        Pattern::Bind(name) => MatchPattern::Bind(name.clone()),
        Pattern::Ctor { name, args } => MatchPattern::Ctor {
            name: name.clone(),
            args: args.iter().map(pattern_to_match_pattern).collect(),
        },
    }
}

fn runtime_call(name: &str, args: Vec<tsast::Expr>) -> tsast::Expr {
    tsast::Expr::Call {
        callee: Box::new(tsast::Expr::Ident(name.to_owned())),
        args,
    }
}
