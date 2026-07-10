//! File discovery and classification. The suffix declares the role (D-03);
//! a language split across files uses the first name segment.

use std::io;
use std::path::Path;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FileKind {
    /// `lumo.langue` — no kind suffix; pipe glue and entry point (D-27).
    Manifest,
    /// `Lumo.expr.syn.langue` — feeds language `Lumo`.
    Syn { language: String },
    /// Parsed from M1 on.
    Elab,
    /// Parsed from M2 on.
    Type,
}

/// Classify a file name. `None` = not a `.langue` file.
pub fn classify_file_name(name: &str) -> Option<FileKind> {
    let segments: Vec<&str> = name.split('.').collect();
    if segments.last() != Some(&"langue") || segments.len() < 2 {
        return None;
    }
    match segments[segments.len() - 2] {
        "syn" if segments.len() >= 3 => Some(FileKind::Syn { language: segments[0].to_owned() }),
        "elab" if segments.len() >= 3 => Some(FileKind::Elab),
        "type" if segments.len() >= 3 => Some(FileKind::Type),
        _ if segments.len() == 2 => Some(FileKind::Manifest),
        // e.g. `foo.bar.langue` — dotted but no known kind.
        _ => None,
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoadedFile {
    /// Path relative to the project root, `/`-separated.
    pub path: String,
    pub kind: FileKind,
    pub text: String,
}

/// Collect every `.langue` file under `root`, sorted by path so every
/// downstream artifact is deterministic.
pub fn scan_project(root: &Path) -> io::Result<Vec<LoadedFile>> {
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<LoadedFile>) -> io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(kind) = classify_file_name(name) {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(LoadedFile { path: rel, kind, text: std::fs::read_to_string(&path)? });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_kinds() {
        assert_eq!(classify_file_name("lumo.langue"), Some(FileKind::Manifest));
        assert_eq!(
            classify_file_name("Lumo.tokens.syn.langue"),
            Some(FileKind::Syn { language: "Lumo".into() })
        );
        assert_eq!(
            classify_file_name("Lumo.syn.langue"),
            Some(FileKind::Syn { language: "Lumo".into() })
        );
        assert_eq!(classify_file_name("item.elab.langue"), Some(FileKind::Elab));
        assert_eq!(classify_file_name("fn.type.langue"), Some(FileKind::Type));
        assert_eq!(classify_file_name("notes.md"), None);
        assert_eq!(classify_file_name("weird.thing.langue"), None);
    }
}
