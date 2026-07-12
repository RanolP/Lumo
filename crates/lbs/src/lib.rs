//! lbs slice 1 (D-53): manifest-driven whole-program assembly. Packages
//! (`lumo.toml`) topo-sort over `[deps]`, modules concatenate in binding
//! order (the judge binds top-down), platform halves (`src#{suffix}/`)
//! merge per module. The assembled unit feeds `lumo-syntax`'s drivers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub root: PathBuf,
    /// Target specs this package supports; the first is the default.
    pub targets: Vec<String>,
    /// Explicit module binding order; `None` = alphabetical disk scan.
    pub modules: Option<Vec<String>>,
    /// Host-binding file prepended to bin outputs (topo order).
    pub js_prelude: Option<PathBuf>,
    /// Declared order matters: it drives the topo DFS.
    pub deps: Vec<(String, PathBuf)>,
}

pub fn parse_manifest(content: &str, root: &Path) -> Result<Manifest, String> {
    let mut name = None;
    let mut targets = Vec::new();
    let mut modules = None;
    let mut js_prelude = None;
    let mut deps = Vec::new();
    let mut section = "";

    for (line_no, raw) in content.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line;
            continue;
        }
        let Some(eq) = line.find('=') else {
            return Err(format!("line {}: expected `key = value`: {line}", line_no + 1));
        };
        let key = line[..eq].trim();
        let value = line[eq + 1..].trim();
        match section {
            "[package]" => match key {
                "name" => name = Some(unquote(value).to_owned()),
                "targets" => {
                    targets = parse_string_array(value)
                        .ok_or_else(|| format!("line {}: targets must be a string array", line_no + 1))?;
                }
                "modules" => {
                    modules = Some(parse_string_array(value).ok_or_else(|| {
                        format!("line {}: modules must be a string array", line_no + 1)
                    })?);
                }
                "js-prelude" => js_prelude = Some(root.join(unquote(value))),
                _ => return Err(format!("line {}: unknown package key: {key}", line_no + 1)),
            },
            "[deps]" => deps.push((key.to_owned(), root.join(unquote(value)))),
            "" => return Err(format!("line {}: key outside of section: {key}", line_no + 1)),
            _ => return Err(format!("line {}: unknown section: {section}", line_no + 1)),
        }
    }

    Ok(Manifest {
        name: name.ok_or("missing [package] name")?,
        root: root.to_owned(),
        targets,
        modules,
        js_prelude,
        deps,
    })
}

fn unquote(v: &str) -> &str {
    v.strip_prefix('"').and_then(|v| v.strip_suffix('"')).unwrap_or(v)
}

fn parse_string_array(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    inner
        .split(',')
        .map(|part| {
            let part = part.trim();
            Some(part.strip_prefix('"')?.strip_suffix('"')?.to_owned())
        })
        .collect()
}

pub fn load_manifest(dir: &Path) -> Result<Manifest, String> {
    let path = dir.join("lumo.toml");
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_manifest(&content, dir)
}

/// Walk up from `start` to the nearest directory holding a `lumo.toml`.
pub fn find_package_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_owned();
    loop {
        if dir.join("lumo.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Directory suffixes a target spec enables, in merge order:
/// `"js.node"` → `["js", "js.node"]` (legacy lbs rule).
pub fn target_suffixes(spec: &str) -> Vec<String> {
    let parts: Vec<&str> = spec.split('.').collect();
    (1..=parts.len()).map(|n| parts[..n].join(".")).collect()
}

/// The target to build: `--target` flag, else the manifest's first
/// entry, else `js`. Only the `js` base exists in this slice.
pub fn resolve_target(manifest: &Manifest, flag: Option<&str>) -> Result<String, String> {
    let spec = flag
        .map(str::to_owned)
        .or_else(|| manifest.targets.first().cloned())
        .unwrap_or_else(|| "js".to_owned());
    if let Some(flag) = flag {
        if !manifest.targets.is_empty() && !manifest.targets.iter().any(|t| t == flag) {
            return Err(format!(
                "target `{flag}` is not in manifest targets {:?}",
                manifest.targets
            ));
        }
    }
    match spec.split('.').next() {
        Some("js") => Ok(spec),
        _ => Err(format!("unknown target `{spec}` (supported base: js)")),
    }
}

/// One source file's byte range inside the assembled unit.
#[derive(Debug, Clone)]
pub struct Part {
    pub file: PathBuf,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug)]
pub struct Unit {
    pub text: String,
    pub parts: Vec<Part>,
    /// Dep `js-prelude` files in topo order (root package last).
    pub preludes: Vec<PathBuf>,
    /// The root package; `is_bin` when it has a `main` module.
    pub name: String,
    pub is_bin: bool,
}

/// Assemble the whole-program unit for the package at `root_dir`.
pub fn assemble(root_dir: &Path, target_spec: &str) -> Result<Unit, String> {
    let suffixes = target_suffixes(target_spec);
    let mut order: Vec<Manifest> = Vec::new();
    let mut names: HashMap<String, PathBuf> = HashMap::new();
    let mut visiting: Vec<PathBuf> = Vec::new();
    visit(root_dir, &mut order, &mut names, &mut visiting)?;

    let mut text = String::new();
    let mut parts = Vec::new();
    let mut preludes = Vec::new();
    let mut is_bin = false;
    let root_name = order.last().expect("root package is always visited").name.clone();

    for pkg in &order {
        if let Some(p) = &pkg.js_prelude {
            preludes.push(p.clone());
        }
        let modules = match &pkg.modules {
            Some(m) => m.clone(),
            None => scan_modules(&pkg.root, &suffixes),
        };
        for module in &modules {
            if pkg.name == root_name && module == "main" {
                is_bin = true;
            }
            let mut found = false;
            let mut candidates = vec![pkg.root.join("src").join(format!("{module}.lumo"))];
            for s in &suffixes {
                candidates.push(pkg.root.join(format!("src#{s}")).join(format!("{module}.lumo")));
            }
            for path in candidates {
                let Ok(source) = std::fs::read_to_string(&path) else { continue };
                found = true;
                if !text.is_empty() {
                    text.push('\n');
                }
                let start = text.len();
                text.push_str(&source);
                parts.push(Part { file: path, start, end: text.len() });
            }
            if !found {
                return Err(format!(
                    "package `{}`: module `{module}` has no source for target `{target_spec}`",
                    pkg.name
                ));
            }
        }
    }

    Ok(Unit { text, parts, preludes, name: root_name, is_bin })
}

fn visit(
    dir: &Path,
    order: &mut Vec<Manifest>,
    names: &mut HashMap<String, PathBuf>,
    visiting: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let root = dir
        .canonicalize()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    if visiting.contains(&root) {
        return Err(format!("dependency cycle through {}", root.display()));
    }
    let manifest = load_manifest(&root)?;
    match names.get(&manifest.name) {
        Some(prev) if *prev == root => return Ok(()), // already assembled
        Some(prev) => {
            return Err(format!(
                "two packages named `{}`: {} and {}",
                manifest.name,
                prev.display(),
                root.display()
            ));
        }
        None => {}
    }
    visiting.push(root.clone());
    for (_, dep_path) in &manifest.deps {
        visit(dep_path, order, names, visiting)?;
    }
    visiting.pop();
    names.insert(manifest.name.clone(), root);
    order.push(manifest);
    Ok(())
}

/// Module names on disk (common + platform dirs), alphabetical.
fn scan_modules(pkg_root: &Path, suffixes: &[String]) -> Vec<String> {
    let mut dirs = vec![pkg_root.join("src")];
    for s in suffixes {
        dirs.push(pkg_root.join(format!("src#{s}")));
    }
    let mut modules: Vec<String> = Vec::new();
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("lumo") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if !modules.iter().any(|m| m == stem) {
                    modules.push(stem.to_owned());
                }
            }
        }
    }
    modules.sort();
    modules
}

/// Map a byte offset in the assembled unit to `file:line:col` (1-based).
pub fn locate(unit: &Unit, offset: usize) -> Option<String> {
    let part = unit.parts.iter().find(|p| p.start <= offset && offset < p.end.max(p.start + 1))?;
    let local = offset - part.start;
    let slice = &unit.text[part.start..part.start + local];
    let line = slice.matches('\n').count() + 1;
    let col = local - slice.rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
    Some(format!("{}:{line}:{col}", part.file.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parses_all_keys() {
        let m = parse_manifest(
            r#"
[package]
name = "libcore"                # trailing comment
targets = ["js.node", "js"]
modules = ["prelude", "cmp"]
js-prelude = "js/prelude.js"

[deps]
libstd = "../libstd"
other = "../other"
"#,
            Path::new("/pkg"),
        )
        .unwrap();
        assert_eq!(m.name, "libcore");
        assert_eq!(m.targets, ["js.node", "js"]);
        assert_eq!(m.modules.as_deref(), Some(&["prelude".to_owned(), "cmp".to_owned()][..]));
        assert_eq!(m.js_prelude.as_deref(), Some(Path::new("/pkg/js/prelude.js")));
        assert_eq!(m.deps[0].0, "libstd");
        assert_eq!(m.deps[1].1, Path::new("/pkg/../other"));
    }

    #[test]
    fn suffixes_expand_dotted_prefixes() {
        assert_eq!(target_suffixes("js"), ["js"]);
        assert_eq!(target_suffixes("js.node"), ["js", "js.node"]);
    }

    #[test]
    fn locate_maps_into_the_second_part() {
        let unit = Unit {
            text: "abc\ndef\nghi".to_owned(),
            parts: vec![
                Part { file: "a.lumo".into(), start: 0, end: 3 },
                Part { file: "b.lumo".into(), start: 4, end: 11 },
            ],
            preludes: vec![],
            name: "t".into(),
            is_bin: false,
        };
        assert_eq!(locate(&unit, 8).unwrap(), "b.lumo:2:1");
    }
}
