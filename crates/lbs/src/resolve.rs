use std::collections::HashMap;
use std::path::PathBuf;

/// Filesystem-based module resolver for `compile_with_deps`.
///
/// Maps use-paths like `["libcore", "io"]` to `(filename, source)` pairs
/// by looking up the package name in the deps table and reading the file.
/// For each target suffix (in order, e.g. `["js", "js.node"]`), platform
/// sources in `src#{suffix}/` are appended after the common `src/` source.
pub struct FsResolver {
    deps: HashMap<String, PathBuf>,
    /// Ordered list of directory suffixes to merge, e.g. `["js", "js.node"]`.
    target_suffixes: Vec<String>,
    cache: HashMap<String, (String, String)>,
}

impl FsResolver {
    pub fn new(deps: HashMap<String, PathBuf>, target_suffixes: Vec<String>) -> Self {
        Self {
            deps,
            target_suffixes,
            cache: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, path: &[String]) -> Option<(String, String)> {
        if path.len() < 2 {
            return None;
        }

        let pkg = &path[0];
        let module = &path[1];
        let canonical_name = format!("{pkg}/{module}.lumo");

        if let Some(cached) = self.cache.get(&canonical_name) {
            return Some(cached.clone());
        }

        let dep_path = self.deps.get(pkg)?;

        let common_path = dep_path.join("src").join(format!("{module}.lumo"));
        let common_source = std::fs::read_to_string(&common_path).ok();

        let mut platform_sources: Vec<String> = Vec::new();
        for suffix in &self.target_suffixes {
            let platform_path = dep_path
                .join(format!("src#{suffix}"))
                .join(format!("{module}.lumo"));
            if let Ok(src) = std::fs::read_to_string(&platform_path) {
                platform_sources.push(src);
            }
        }

        let source = match (common_source, platform_sources.is_empty()) {
            (Some(common), true) => common,
            (Some(common), false) => format!("{common}\n{}", platform_sources.join("\n")),
            (None, false) => platform_sources.join("\n"),
            (None, true) => return None,
        };

        let entry = (canonical_name.clone(), source);
        self.cache.insert(canonical_name, entry.clone());
        Some(entry)
    }
}

/// Create a resolver closure suitable for `QueryEngine::compile_with_deps`.
pub fn make_resolver(
    deps: HashMap<String, PathBuf>,
    target_suffixes: Vec<String>,
) -> impl FnMut(&[String]) -> Option<(String, String)> {
    let mut resolver = FsResolver::new(deps, target_suffixes);
    move |path: &[String]| resolver.resolve(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_existing_module() {
        let tmp = std::env::temp_dir().join("lbs_test_resolve");
        let src = tmp.join("src");
        let _ = fs::create_dir_all(&src);
        fs::write(src.join("io.lumo"), "extern fn println(msg: String);").unwrap();

        let mut deps = HashMap::new();
        deps.insert("libstd".to_owned(), tmp.clone());

        let mut resolver = FsResolver::new(deps, vec!["js".into()]);
        let result = resolver.resolve(&["libstd".into(), "io".into()]);
        assert!(result.is_some());
        let (name, source) = result.unwrap();
        assert_eq!(name, "libstd/io.lumo");
        assert!(source.contains("println"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn returns_none_for_unknown_package() {
        let mut resolver = FsResolver::new(HashMap::new(), vec!["js".into()]);
        assert!(resolver
            .resolve(&["unknown".into(), "mod".into()])
            .is_none());
    }
}
