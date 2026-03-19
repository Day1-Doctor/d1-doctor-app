use std::path::{Path, PathBuf};

use super::McpTool;

/// MCP tool server interface for filesystem operations.
///
/// All operations are sandboxed to a configured workspace root directory.
/// Attempts to access paths outside the workspace are rejected.
pub struct FilesystemTool {
    workspace_root: PathBuf,
}

impl FilesystemTool {
    /// Create a new `FilesystemTool` scoped to the given workspace root.
    ///
    /// All file operations will be restricted to paths within this directory.
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Validate that the resolved path is within the workspace root.
    ///
    /// Returns the canonicalized path on success, or an error if the path
    /// escapes the workspace boundary.
    fn validate_path(&self, path: &str) -> Result<PathBuf, String> {
        let target = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.workspace_root.join(path)
        };

        // Canonicalize the workspace root (it must exist).
        let root_canonical = self
            .workspace_root
            .canonicalize()
            .map_err(|e| format!("cannot resolve workspace root: {e}"))?;

        // If the target already exists, canonicalize it directly.
        // Otherwise walk up the path to find the nearest existing ancestor,
        // canonicalize that, and re-append the remaining components.
        let target_canonical = if target.exists() {
            target
                .canonicalize()
                .map_err(|e| format!("cannot resolve path: {e}"))?
        } else {
            // Collect path components that don't yet exist on disk.
            let mut suffix_parts: Vec<&std::ffi::OsStr> = Vec::new();
            let mut ancestor = target.as_path();
            loop {
                if ancestor.exists() {
                    break;
                }
                if let Some(file_name) = ancestor.file_name() {
                    suffix_parts.push(file_name);
                    ancestor = ancestor
                        .parent()
                        .unwrap_or(ancestor);
                } else {
                    // Reached filesystem root without finding an existing dir.
                    return Err(format!(
                        "cannot resolve any ancestor of '{}'",
                        target.display()
                    ));
                }
            }
            let mut resolved = ancestor
                .canonicalize()
                .map_err(|e| format!("cannot resolve ancestor path: {e}"))?;
            // Re-append in reverse order (we collected bottom-up).
            for part in suffix_parts.into_iter().rev() {
                resolved.push(part);
            }
            resolved
        };

        if !target_canonical.starts_with(&root_canonical) {
            return Err(format!(
                "path '{}' is outside the workspace root '{}'",
                target_canonical.display(),
                root_canonical.display()
            ));
        }

        Ok(target_canonical)
    }

    /// Read a file and return its contents as a string.
    pub fn read(&self, path: &str) -> Result<String, String> {
        let validated = self.validate_path(path)?;
        std::fs::read_to_string(&validated)
            .map_err(|e| format!("read '{}': {e}", validated.display()))
    }

    /// Write content to a file, creating it if it doesn't exist.
    pub fn write(&self, path: &str, content: &str) -> Result<(), String> {
        let validated = self.validate_path(path)?;
        // Ensure parent directories exist.
        if let Some(parent) = validated.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create directories: {e}"))?;
        }
        std::fs::write(&validated, content)
            .map_err(|e| format!("write '{}': {e}", validated.display()))
    }

    /// Search for files matching a glob pattern relative to the workspace root.
    ///
    /// Returns a list of matching paths relative to the workspace root.
    pub fn glob(&self, pattern: &str) -> Result<Vec<String>, String> {
        let root_canonical = self
            .workspace_root
            .canonicalize()
            .map_err(|e| format!("cannot resolve workspace root: {e}"))?;

        // Build absolute glob pattern anchored to the workspace root.
        let abs_pattern = if Path::new(pattern).is_absolute() {
            pattern.to_string()
        } else {
            format!("{}/{}", root_canonical.display(), pattern)
        };

        let entries =
            glob::glob(&abs_pattern).map_err(|e| format!("invalid glob pattern: {e}"))?;

        let mut results = Vec::new();
        for entry in entries {
            match entry {
                Ok(path) => {
                    let canonical = path
                        .canonicalize()
                        .unwrap_or_else(|_| path.clone());
                    // Only include paths within the workspace.
                    if canonical.starts_with(&root_canonical) {
                        if let Ok(relative) = canonical.strip_prefix(&root_canonical) {
                            results.push(relative.display().to_string());
                        } else {
                            results.push(canonical.display().to_string());
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("glob entry error: {e}");
                }
            }
        }

        Ok(results)
    }

    /// List the contents of a directory.
    ///
    /// Returns a list of entry names (not full paths).
    pub fn list_dir(&self, path: &str) -> Result<Vec<String>, String> {
        let validated = self.validate_path(path)?;

        if !validated.is_dir() {
            return Err(format!("'{}' is not a directory", validated.display()));
        }

        let entries = std::fs::read_dir(&validated)
            .map_err(|e| format!("read directory '{}': {e}", validated.display()))?;

        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("read entry: {e}"))?;
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }

        names.sort();
        Ok(names)
    }
}

impl McpTool for FilesystemTool {
    fn name(&self) -> &str {
        "filesystem"
    }

    fn risk_level(&self) -> &str {
        "medium"
    }

    fn description(&self) -> &str {
        "Read, write, glob, and list files within the workspace directory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_workspace() -> (tempfile::TempDir, FilesystemTool) {
        let dir = tempfile::tempdir().expect("create temp dir");
        let tool = FilesystemTool::new(dir.path().to_path_buf());
        (dir, tool)
    }

    #[test]
    fn test_read_write_roundtrip() {
        let (dir, tool) = setup_workspace();
        let file_path = dir.path().join("hello.txt");
        let path_str = file_path.to_str().unwrap();

        tool.write(path_str, "Hello, world!").unwrap();
        let content = tool.read(path_str).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_write_creates_parent_dirs() {
        let (dir, tool) = setup_workspace();
        let file_path = dir.path().join("sub").join("dir").join("file.txt");
        let path_str = file_path.to_str().unwrap();

        tool.write(path_str, "nested content").unwrap();
        let content = tool.read(path_str).unwrap();
        assert_eq!(content, "nested content");
    }

    #[test]
    fn test_read_nonexistent_file_returns_error() {
        let (_dir, tool) = setup_workspace();
        let result = tool.read("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_path_escape_rejected() {
        let (dir, tool) = setup_workspace();
        // Create a file outside the workspace.
        let outside = std::env::temp_dir().join("d1d-outside-test.txt");
        std::fs::write(&outside, "secret").ok();

        let result = tool.read(outside.to_str().unwrap());
        assert!(result.is_err(), "reading outside workspace should fail");

        // Also test relative traversal.
        let traversal = format!("{}/../../../etc/passwd", dir.path().display());
        let result = tool.read(&traversal);
        assert!(result.is_err(), "path traversal should be rejected");

        // Cleanup
        std::fs::remove_file(&outside).ok();
    }

    #[test]
    fn test_glob_finds_files() {
        let (dir, tool) = setup_workspace();

        // Create some files.
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        std::fs::write(dir.path().join("b.txt"), "b").unwrap();
        std::fs::write(dir.path().join("c.rs"), "c").unwrap();

        let txt_files = tool.glob("*.txt").unwrap();
        assert_eq!(txt_files.len(), 2);
        assert!(txt_files.iter().any(|f| f.contains("a.txt")));
        assert!(txt_files.iter().any(|f| f.contains("b.txt")));

        let all_files = tool.glob("*").unwrap();
        assert!(all_files.len() >= 3);
    }

    #[test]
    fn test_glob_with_subdirs() {
        let (dir, tool) = setup_workspace();

        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src").join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("src").join("lib.rs"), "// lib").unwrap();

        let results = tool.glob("src/*.rs").unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_list_dir() {
        let (dir, tool) = setup_workspace();

        std::fs::write(dir.path().join("file1.txt"), "").unwrap();
        std::fs::write(dir.path().join("file2.txt"), "").unwrap();
        std::fs::create_dir(dir.path().join("subdir")).unwrap();

        let entries = tool.list_dir(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&"file1.txt".to_string()));
        assert!(entries.contains(&"file2.txt".to_string()));
        assert!(entries.contains(&"subdir".to_string()));
    }

    #[test]
    fn test_list_dir_on_file_returns_error() {
        let (dir, tool) = setup_workspace();
        let file_path = dir.path().join("notadir.txt");
        std::fs::write(&file_path, "").unwrap();

        let result = tool.list_dir(file_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not a directory"));
    }

    #[test]
    fn test_relative_path_resolved_within_workspace() {
        let (dir, tool) = setup_workspace();
        std::fs::write(dir.path().join("rel.txt"), "relative content").unwrap();

        let content = tool.read("rel.txt").unwrap();
        assert_eq!(content, "relative content");
    }

    #[test]
    fn test_mcp_trait() {
        let tool = FilesystemTool::new(PathBuf::from("/tmp"));
        assert_eq!(tool.name(), "filesystem");
        assert_eq!(tool.risk_level(), "medium");
        assert!(!tool.description().is_empty());
    }
}
