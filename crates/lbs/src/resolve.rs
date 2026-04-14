use std::collections::HashMap;
use std::path::PathBuf;

/// Filesystem-based module resolver for `compile_with_deps`.
///
/// Maps use-paths like `["libcore", "io"]` to `(filename, source)` pairs
/// by looking up the package name in the deps table and reading the file.
/// Platform-specific source sets (`src#{target}/`) are checked first,
/// falling back to `src/`.
pub struct FsResolver {
    deps: HashMap<String, PathBuf>,
    /// Target suffix for platform source sets (e.g. "js", "rs").
    target: String,
    cache: HashMap<String, (String, String)>,
}

impl FsResolver {
    pub fn new(deps: HashMap<String, PathBuf>, target: &str) -> Self {
        Self {
            deps,
            target: target.to_owned(),
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

        // Load common and platform sources, merge if both exist
        let common_path = dep_path.join("src").join(format!("{module}.lumo"));
        let platform_path = dep_path
            .join(format!("src#{}", self.target))
            .join(format!("{module}.lumo"));

        let common_source = std::fs::read_to_string(&common_path).ok();
        let platform_source = std::fs::read_to_string(&platform_path).ok();

        let source = match (common_source, platform_source) {
            (Some(common), Some(platform)) => format!("{common}\n{platform}"),
            (Some(common), None) => common,
            (None, Some(platform)) => platform,
            (None, None) => return None,
        };

        let entry = (canonical_name.clone(), source);
        self.cache.insert(canonical_name, entry.clone());
        Some(entry)
    }
}

/// Create a resolver closure suitable for `QueryEngine::compile_with_deps`.
pub fn make_resolver(
    deps: HashMap<String, PathBuf>,
    target: &str,
) -> impl FnMut(&[String]) -> Option<(String, String)> {
    let mut resolver = FsResolver::new(deps, target);
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

        let mut resolver = FsResolver::new(deps, "js");
        let result = resolver.resolve(&["libstd".into(), "io".into()]);
        assert!(result.is_some());
        let (name, source) = result.unwrap();
        assert_eq!(name, "libstd/io.lumo");
        assert!(source.contains("println"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn returns_none_for_unknown_package() {
        let mut resolver = FsResolver::new(HashMap::new(), "js");
        assert!(resolver
            .resolve(&["unknown".into(), "mod".into()])
            .is_none());
    }
}
