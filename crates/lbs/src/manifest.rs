use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// Binary: has `main.lumo`
    Bin(PathBuf),
    /// Library: has `lib.lumo`
    Lib(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub name: String,
    pub entry: EntryKind,
    pub out_dir: PathBuf,
    pub deps: HashMap<String, PathBuf>,
}

pub fn parse(content: &str, project_root: &Path) -> Result<Manifest, String> {
    let mut name = None;
    let mut out_dir = None;
    let mut deps = HashMap::new();
    let mut current_section = "";

    for (line_no, raw_line) in content.lines().enumerate() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            current_section = raw_line.trim();
            continue;
        }

        let Some((key, value)) = parse_key_value(line) else {
            return Err(format!("line {}: invalid syntax: {line}", line_no + 1));
        };

        match current_section {
            "[package]" => match key {
                "name" => name = Some(value.to_owned()),
                "out-dir" => out_dir = Some(value.to_owned()),
                _ => return Err(format!("line {}: unknown package key: {key}", line_no + 1)),
            },
            "[deps]" => {
                deps.insert(key.to_owned(), project_root.join(value));
            }
            "" => return Err(format!("line {}: key outside of section: {key}", line_no + 1)),
            _ => return Err(format!("line {}: unknown section: {current_section}", line_no + 1)),
        }
    }

    let name = name.ok_or("missing [package] name")?;
    let out_dir = out_dir.unwrap_or_else(|| "dist".to_owned());
    let entry = detect_entry(project_root)?;

    Ok(Manifest {
        name,
        entry,
        out_dir: project_root.join(out_dir),
        deps,
    })
}

fn detect_entry(project_root: &Path) -> Result<EntryKind, String> {
    let src = project_root.join("src");
    let main_lumo = src.join("main.lumo");
    let lib_lumo = src.join("lib.lumo");

    match (main_lumo.exists(), lib_lumo.exists()) {
        (true, true) => Err("both src/main.lumo and src/lib.lumo exist; pick one".into()),
        (true, false) => Ok(EntryKind::Bin(main_lumo)),
        (false, true) => Ok(EntryKind::Lib(lib_lumo)),
        (false, false) => Err("no src/main.lumo or src/lib.lumo found".into()),
    }
}

fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let eq = line.find('=')?;
    let key = line[..eq].trim();
    let value = line[eq + 1..].trim();
    let value = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value);
    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_main_entry() {
        let tmp = std::env::temp_dir().join("lbs_test_main_entry");
        let src = tmp.join("src");
        let _ = fs::create_dir_all(&src);
        fs::write(src.join("main.lumo"), "fn main() := produce 1").unwrap();

        let content = "[package]\nname = \"test\"\n";
        let m = parse(content, &tmp).unwrap();
        assert_eq!(m.entry, EntryKind::Bin(src.join("main.lumo")));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn detects_lib_entry() {
        let tmp = std::env::temp_dir().join("lbs_test_lib_entry");
        let src = tmp.join("src");
        let _ = fs::create_dir_all(&src);
        fs::write(src.join("lib.lumo"), "fn hello() := produce 1").unwrap();

        let content = "[package]\nname = \"mylib\"\n";
        let m = parse(content, &tmp).unwrap();
        assert_eq!(m.entry, EntryKind::Lib(src.join("lib.lumo")));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn errors_when_no_entry() {
        let tmp = std::env::temp_dir().join("lbs_test_no_entry");
        let _ = fs::create_dir_all(&tmp);

        let content = "[package]\nname = \"empty\"\n";
        let result = parse(content, &tmp);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no src/main.lumo or src/lib.lumo"));

        let _ = fs::remove_dir_all(&tmp);
    }
}
