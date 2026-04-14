use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    diagnostics::Diagnostic,
    hir,
    lexer::Span,
    lir, lst,
    parser::{self, ParseOutput},
    typecheck,
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
    /// then lowers the merged HIR to a single LIR.
    pub fn lower_module(&mut self, files: &[&str]) -> Option<lir::File> {
        let mut hir_files = Vec::new();
        for file in files {
            hir_files.push(self.lower_hir(file)?);
        }
        let merged = hir::merge_files(&hir_files);
        Some(lir::lower(&merged))
    }

    /// Compile entry files with transitive `use` resolution.
    ///
    /// The `resolve` callback maps a use-path (e.g. `["lumo_std", "io"]`) to
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
        let mut all_files: HashSet<String> = entry_files.iter().map(|f| f.to_string()).collect();
        let mut pending: VecDeque<String> = all_files.iter().cloned().collect();

        while let Some(file) = pending.pop_front() {
            let parsed = self.parse(&file)?;
            for use_path in collect_use_paths(&parsed.file) {
                if let Some((filename, source)) = resolve(&use_path) {
                    if all_files.insert(filename.clone()) {
                        self.set_file(&filename, source);
                        pending.push_back(filename);
                    }
                }
            }
        }

        let file_refs: Vec<&str> = all_files.iter().map(|s| s.as_str()).collect();
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

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}
