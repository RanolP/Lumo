//! Corpus fixture harness (D-32, tree-sitter-corpus shape).
//!
//! ```text
//! === title ===
//! :parse(Lumo)          or  :fails(Lumo)
//!
//! <source>
//!
//! ---
//!
//! (FILE (FN_DECL …))
//! ```
//!
//! `:parse(L)` expects a clean parse, the named-node S-expression, and —
//! automatically — parse → canonical print → re-parse tree equality plus
//! byte-exact lossless printing. `:fails(L)` expects at least one error.
//! `LANGC_UPDATE=1` blesses expected blocks in place.

use std::path::{Path, PathBuf};

use langue_rt::{ElabReport, ParseReport};

pub type ReportFn = fn(&str) -> ParseReport;
pub type ElabFn = fn(&str) -> ElabReport;
/// `:infer(L)` reuses the ElabReport shape: `output` is the
/// `name : Type` lines, or `ERROR: …` when judging bails (D-26).
pub type InferFn = fn(&str) -> ElabReport;
/// `:optimize(L)` reuses the ElabReport shape: `output` is the
/// optimized L-text, or `ERROR: …` (unencodable input, non-converging
/// reduction; D-42).
pub type OptimizeFn = fn(&str) -> ElabReport;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Attr {
    Parse(String),
    Fails(String),
    /// `:elab(Lumo -> MIR)` — source is A-text, expected is B-text;
    /// comparison canonicalizes both sides with B's printer (D-32).
    Elab { from: String, to: String },
    /// `:infer(Lumo)` — the full pipeline (parse | elab | judge);
    /// expected is `name : Type` lines, or a line starting with
    /// `ERROR` for an expected bail (message free, D-26).
    Infer(String),
    /// `:optimize(MIR)` — same-language saturation + extraction (D-42);
    /// source and expected are both L-text, compared canonicalized like
    /// `:elab`; a line starting with `ERROR` matches any error.
    Optimize(String),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Case {
    pub title: String,
    pub attr: Attr,
    pub source: String,
    pub expected: Option<String>,
    pub line: usize,
}

pub fn parse_corpus(path: &Path, text: &str) -> Result<Vec<Case>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut cases = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].starts_with("===") {
            if !lines[i].trim().is_empty() {
                return Err(format!(
                    "{}:{}: expected `=== title ===` fence",
                    path.display(),
                    i + 1
                ));
            }
            i += 1;
            continue;
        }
        let case_line = i + 1;
        let title = lines[i].trim_matches(['=', ' ']).to_owned();
        i += 1;

        let mut attr = None;
        while i < lines.len() && lines[i].trim_start().starts_with(':') {
            let a = lines[i].trim();
            attr = Some(parse_attr(a).ok_or_else(|| {
                format!("{}:{}: unknown attribute `{a}`", path.display(), i + 1)
            })?);
            i += 1;
        }
        let attr = attr.ok_or_else(|| {
            format!(
                "{}:{}: case `{title}` has no `:parse(L)`/`:fails(L)` attribute",
                path.display(),
                case_line
            )
        })?;

        let mut source_lines = Vec::new();
        while i < lines.len() && lines[i] != "---" && !lines[i].starts_with("===") {
            source_lines.push(lines[i]);
            i += 1;
        }
        let mut expected = None;
        if i < lines.len() && lines[i] == "---" {
            i += 1;
            let mut expected_lines = Vec::new();
            while i < lines.len() && !lines[i].starts_with("===") {
                expected_lines.push(lines[i]);
                i += 1;
            }
            expected = Some(trim_blank(&expected_lines).join("\n"));
        }
        cases.push(Case {
            title,
            attr,
            source: trim_blank(&source_lines).join("\n"),
            expected,
            line: case_line,
        });
    }
    Ok(cases)
}

fn parse_attr(line: &str) -> Option<Attr> {
    let inner = |prefix: &str| {
        line.strip_prefix(prefix)?
            .strip_suffix(')')
            .map(str::to_owned)
    };
    if let Some(lang) = inner(":parse(") {
        return Some(Attr::Parse(lang));
    }
    if let Some(lang) = inner(":fails(") {
        return Some(Attr::Fails(lang));
    }
    if let Some(pair) = inner(":elab(") {
        let (from, to) = pair.split_once("->")?;
        return Some(Attr::Elab { from: from.trim().to_owned(), to: to.trim().to_owned() });
    }
    if let Some(lang) = inner(":infer(") {
        return Some(Attr::Infer(lang));
    }
    if let Some(lang) = inner(":optimize(") {
        return Some(Attr::Optimize(lang));
    }
    None
}

fn trim_blank<'l>(lines: &[&'l str]) -> Vec<&'l str> {
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(lines.len());
    let end = lines.iter().rposition(|l| !l.trim().is_empty()).map_or(start, |e| e + 1);
    lines[start..end].to_vec()
}

fn render_corpus(cases: &[Case]) -> String {
    let mut out = String::new();
    for case in cases {
        out.push_str(&format!("=== {} ===\n", case.title));
        match &case.attr {
            Attr::Parse(l) => out.push_str(&format!(":parse({l})\n")),
            Attr::Fails(l) => out.push_str(&format!(":fails({l})\n")),
            Attr::Elab { from, to } => out.push_str(&format!(":elab({from} -> {to})\n")),
            Attr::Infer(l) => out.push_str(&format!(":infer({l})\n")),
            Attr::Optimize(l) => out.push_str(&format!(":optimize({l})\n")),
        }
        out.push('\n');
        out.push_str(&case.source);
        out.push('\n');
        if let Some(expected) = &case.expected {
            out.push_str("\n---\n\n");
            out.push_str(expected);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

fn normalize(sexpr: &str) -> String {
    sexpr.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Run every `**/*.test` under `root`. `lookup` resolves a language name
/// to its generated parse-report fn and `elab_lookup` an elab pair to
/// its generated elab-report fn (both from the registry);
/// `infer_lookup`/`optimize_lookup` resolve a language to its
/// handwritten driver. Bless with `LANGC_UPDATE=1`. Returns a summary
/// or the combined failures.
pub fn run_dir(
    root: &Path,
    lookup: impl Fn(&str) -> Option<ReportFn>,
    elab_lookup: impl Fn(&str, &str) -> Option<ElabFn>,
    infer_lookup: impl Fn(&str) -> Option<InferFn>,
    optimize_lookup: impl Fn(&str) -> Option<OptimizeFn>,
) -> Result<String, String> {
    let update = std::env::var("LANGC_UPDATE").is_ok_and(|v| v == "1");
    let mut files = Vec::new();
    collect_tests(root, &mut files)
        .map_err(|e| format!("cannot read {}: {e}", root.display()))?;
    files.sort();
    if files.is_empty() {
        return Err(format!("no .test files under {}", root.display()));
    }

    let mut failures = Vec::new();
    let mut total = 0;
    let mut blessed = 0;
    for file in &files {
        let text = std::fs::read_to_string(file)
            .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
        let mut cases = match parse_corpus(file, &text) {
            Ok(cases) => cases,
            Err(e) => {
                failures.push(e);
                continue;
            }
        };
        let mut changed = false;
        for case in &mut cases {
            total += 1;
            let fail =
                |msg: String, failures: &mut Vec<String>| {
                    failures.push(format!(
                        "{}:{}: `{}`: {msg}",
                        file.display(),
                        case.line,
                        case.title
                    ));
                };
            if let Attr::Elab { from, to } = &case.attr {
                let Some(elab_fn) = elab_lookup(from, to) else {
                    fail(format!("no elab from `{from}` to `{to}`"), &mut failures);
                    continue;
                };
                let Some(to_report_fn) = lookup(to) else {
                    fail(format!("unknown language `{to}`"), &mut failures);
                    continue;
                };
                let report = elab_fn(&case.source);
                if !report.errors.is_empty() {
                    fail(format!("elab errors: {:?}", report.errors), &mut failures);
                    continue;
                }
                match (&case.expected, update) {
                    (Some(expected), _) => {
                        // Canonicalize-then-compare: the expected block
                        // doubles as a B-language round-trip test (D-32).
                        let exp = to_report_fn(expected);
                        if !exp.errors.is_empty() {
                            fail(
                                format!(
                                    "expected block does not parse as {to}: {:?}",
                                    exp.errors
                                ),
                                &mut failures,
                            );
                        } else if exp.canonical != report.output {
                            fail(
                                format!(
                                    "elab output mismatch:\n  expected: {}\n  actual:   {}",
                                    exp.canonical, report.output
                                ),
                                &mut failures,
                            );
                        }
                    }
                    (None, true) => {
                        case.expected = Some(report.output.clone());
                        changed = true;
                        blessed += 1;
                    }
                    (None, false) => {
                        fail(
                            "missing expected block — bless with LANGC_UPDATE=1".to_owned(),
                            &mut failures,
                        );
                    }
                }
                continue;
            }
            if let Attr::Infer(lang) = &case.attr {
                let Some(infer_fn) = infer_lookup(lang) else {
                    fail(format!("no infer driver for `{lang}`"), &mut failures);
                    continue;
                };
                let report = infer_fn(&case.source);
                if !report.errors.is_empty() {
                    fail(format!("pipeline errors: {:?}", report.errors), &mut failures);
                    continue;
                }
                match (&case.expected, update) {
                    (Some(expected), _) => {
                        // An expected bail matches any bail: messages
                        // are deliberately generic for now (D-26).
                        let both_bail = expected.starts_with("ERROR")
                            && report.output.starts_with("ERROR");
                        if !both_bail && expected.trim() != report.output.trim() {
                            fail(
                                format!(
                                    "inferred types mismatch:\n  expected: {}\n  actual:   {}",
                                    expected.trim(),
                                    report.output.trim()
                                ),
                                &mut failures,
                            );
                        }
                    }
                    (None, true) => {
                        case.expected = Some(report.output.clone());
                        changed = true;
                        blessed += 1;
                    }
                    (None, false) => {
                        fail(
                            "missing expected block — bless with LANGC_UPDATE=1".to_owned(),
                            &mut failures,
                        );
                    }
                }
                continue;
            }
            if let Attr::Optimize(lang) = &case.attr {
                let Some(optimize_fn) = optimize_lookup(lang) else {
                    fail(format!("no optimize driver for `{lang}`"), &mut failures);
                    continue;
                };
                let Some(report_fn) = lookup(lang) else {
                    fail(format!("unknown language `{lang}`"), &mut failures);
                    continue;
                };
                let report = optimize_fn(&case.source);
                if !report.errors.is_empty() {
                    fail(format!("pipeline errors: {:?}", report.errors), &mut failures);
                    continue;
                }
                match (&case.expected, update) {
                    (Some(expected), _) => {
                        // An expected error matches any error (D-26).
                        if expected.starts_with("ERROR") {
                            if !report.output.starts_with("ERROR") {
                                fail(
                                    format!(
                                        "expected an ERROR, got: {}",
                                        report.output.trim()
                                    ),
                                    &mut failures,
                                );
                            }
                        } else {
                            // Canonicalize-then-compare: the expected
                            // block doubles as an L round-trip (D-32).
                            let exp = report_fn(expected);
                            if !exp.errors.is_empty() {
                                fail(
                                    format!(
                                        "expected block does not parse as {lang}: {:?}",
                                        exp.errors
                                    ),
                                    &mut failures,
                                );
                            } else if exp.canonical != report.output {
                                fail(
                                    format!(
                                        "optimize output mismatch:\n  expected: {}\n  actual:   {}",
                                        exp.canonical, report.output
                                    ),
                                    &mut failures,
                                );
                            }
                        }
                    }
                    (None, true) => {
                        case.expected = Some(report.output.clone());
                        changed = true;
                        blessed += 1;
                    }
                    (None, false) => {
                        fail(
                            "missing expected block — bless with LANGC_UPDATE=1".to_owned(),
                            &mut failures,
                        );
                    }
                }
                continue;
            }
            let lang = match &case.attr {
                Attr::Parse(l) | Attr::Fails(l) => l.clone(),
                Attr::Elab { .. } | Attr::Infer(_) | Attr::Optimize(_) => {
                    unreachable!("handled above")
                }
            };
            let Some(report_fn) = lookup(&lang) else {
                fail(format!("unknown language `{lang}`"), &mut failures);
                continue;
            };
            let report = report_fn(&case.source);
            match &case.attr {
                Attr::Elab { .. } | Attr::Infer(_) | Attr::Optimize(_) => {
                    unreachable!("handled above")
                }
                Attr::Fails(_) => {
                    if report.errors.is_empty() {
                        fail("expected parse errors, got a clean parse".to_owned(), &mut failures);
                    }
                }
                Attr::Parse(_) => {
                    if !report.errors.is_empty() {
                        fail(format!("parse errors: {:?}", report.errors), &mut failures);
                        continue;
                    }
                    if report.lossless != case.source {
                        fail("lossless print differs from source".to_owned(), &mut failures);
                    }
                    if report.round_trip_sexpr != report.sexpr {
                        fail(
                            format!(
                                "canonical round-trip changed the tree:\n  canonical: {}\n  before: {}\n  after:  {}",
                                report.canonical, report.sexpr, report.round_trip_sexpr
                            ),
                            &mut failures,
                        );
                    }
                    match (&case.expected, update) {
                        (Some(expected), false) => {
                            if normalize(expected) != normalize(&report.sexpr) {
                                fail(
                                    format!(
                                        "tree mismatch:\n  expected: {}\n  actual:   {}",
                                        normalize(expected),
                                        report.sexpr
                                    ),
                                    &mut failures,
                                );
                            }
                        }
                        (None, false) => {
                            fail(
                                "missing expected block — bless with LANGC_UPDATE=1".to_owned(),
                                &mut failures,
                            );
                        }
                        (_, true) => {
                            if case.expected.as_deref() != Some(report.sexpr.as_str()) {
                                case.expected = Some(report.sexpr.clone());
                                changed = true;
                                blessed += 1;
                            }
                        }
                    }
                }
            }
        }
        if changed {
            std::fs::write(file, render_corpus(&cases))
                .map_err(|e| format!("cannot bless {}: {e}", file.display()))?;
        }
    }

    if failures.is_empty() {
        Ok(format!(
            "{total} case(s) in {} file(s){}",
            files.len(),
            if blessed > 0 { format!(", {blessed} blessed") } else { String::new() }
        ))
    } else {
        Err(format!("{} failure(s):\n{}", failures.len(), failures.join("\n")))
    }
}

fn collect_tests(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_tests(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "test") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cases_and_renders_back() {
        let text = "\
=== simple ===
:parse(Lumo)

fn t() = 1

---

(FILE (FN_DECL (EXPR)))

=== broken ===
:fails(Lumo)

fn (
";
        let cases = parse_corpus(Path::new("x.test"), text).unwrap();
        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].title, "simple");
        assert_eq!(cases[0].attr, Attr::Parse("Lumo".into()));
        assert_eq!(cases[0].source, "fn t() = 1");
        assert_eq!(cases[0].expected.as_deref(), Some("(FILE (FN_DECL (EXPR)))"));
        assert_eq!(cases[1].attr, Attr::Fails("Lumo".into()));
        assert_eq!(cases[1].expected, None);
        // Round-trips through the renderer.
        let rendered = render_corpus(&cases);
        let reparsed = parse_corpus(Path::new("x.test"), &rendered).unwrap();
        assert_eq!(cases, reparsed);
    }

    #[test]
    fn parses_optimize_attr_and_renders_back() {
        let text = "\
=== beta ===
:optimize(MIR)

def t = thunk { force thunk { ret 1 } }

---

def t = thunk { ret 1 }
";
        let cases = parse_corpus(Path::new("x.test"), text).unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].attr, Attr::Optimize("MIR".into()));
        assert_eq!(cases[0].source, "def t = thunk { force thunk { ret 1 } }");
        assert_eq!(cases[0].expected.as_deref(), Some("def t = thunk { ret 1 }"));
        let rendered = render_corpus(&cases);
        let reparsed = parse_corpus(Path::new("x.test"), &rendered).unwrap();
        assert_eq!(cases, reparsed);
    }
}
