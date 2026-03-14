use std::collections::HashMap;

use crate::{
    diagnostics::Diagnostic,
    hir, lir, lst,
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
        let type_errors = typecheck::typecheck_file(&parsed.file);
        diags.extend(type_errors.into_iter().map(|e| Diagnostic {
            start: e.span.start,
            end: e.span.end,
            message: e.message,
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

fn hash_text(text: &str) -> u64 {
    let mut state = 0xcbf29ce484222325_u64;
    for b in text.as_bytes() {
        state ^= *b as u64;
        state = state.wrapping_mul(0x100000001b3);
    }
    state
}

impl Default for QueryEngine {
    fn default() -> Self {
        Self::new()
    }
}
