//! Filesystem operations for the MCP filesystem server.
//!
//! Provides secure file system access with path validation,
//! backup support, and workspace-scoped operations.

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use regex::Regex;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Maximum file size for read operations (1 MB).
const MAX_READ_SIZE: u64 = 1_048_576;

/// Filesystem operations with workspace-scoped path security.
pub struct FilesystemOps {
    /// Root directory for all operations. All paths are validated
    /// to be within this directory.
    workspace_root: PathBuf,
    /// Directory where backups are stored.
    backup_dir: PathBuf,
}

impl FilesystemOps {
    /// Create a new `FilesystemOps` with the given workspace root and backup directory.
    pub fn new(workspace_root: PathBuf, backup_dir: PathBuf) -> Self {
        Self {
            workspace_root,
            backup_dir,
        }
    }

    /// Create a new `FilesystemOps` using default paths:
    /// - workspace_root: user home directory
    /// - backup_dir: `~/.d1doctor/backups/`
    pub fn with_defaults() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot determine home directory"))?;
        let backup_dir = home.join(".d1doctor").join("backups");
        Ok(Self::new(home, backup_dir))
    }

    // ----------------------------------------------------------------
    // Path validation
    // ----------------------------------------------------------------

    /// Resolve and validate that a path is within the workspace root.
    /// Returns the canonicalized absolute path.
    fn validate_path(&self, path: &str) -> Result<PathBuf> {
        let requested = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.workspace_root.join(path)
        };

        // Canonicalize both the workspace root and the requested path.
        // The workspace root must already exist; the requested path may not
        // (e.g., when writing a new file), so we canonicalize the *parent*.
        let canon_root = self
            .workspace_root
            .canonicalize()
            .context("failed to canonicalize workspace root")?;

        // Try to canonicalize the full path first (works if it exists).
        if let Ok(canon) = requested.canonicalize() {
            if canon.starts_with(&canon_root) {
                return Ok(canon);
            }
            bail!(
                "path traversal denied: {} is outside workspace {}",
                canon.display(),
                canon_root.display()
            );
        }

        // Path doesn't exist yet — canonicalize the parent instead.
        let parent = requested
            .parent()
            .ok_or_else(|| anyhow!("path has no parent: {}", requested.display()))?;

        let canon_parent = parent
            .canonicalize()
            .with_context(|| format!("parent directory does not exist: {}", parent.display()))?;

        if !canon_parent.starts_with(&canon_root) {
            bail!(
                "path traversal denied: {} is outside workspace {}",
                canon_parent.display(),
                canon_root.display()
            );
        }

        // Reconstruct the full path with the canonicalized parent.
        let filename = requested
            .file_name()
            .ok_or_else(|| anyhow!("path has no filename: {}", requested.display()))?;
        Ok(canon_parent.join(filename))
    }

    // ----------------------------------------------------------------
    // read_file
    // ----------------------------------------------------------------

    /// Read a file with optional line offset and limit.
    /// Returns content with line numbers prefixed.
    pub fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<String> {
        let abs = self.validate_path(path)?;
        debug!(?abs, "read_file");

        // Check file size.
        let meta = fs::metadata(&abs)
            .with_context(|| format!("file not found: {}", abs.display()))?;
        if meta.len() > MAX_READ_SIZE {
            bail!(
                "file too large: {} bytes (max {} bytes)",
                meta.len(),
                MAX_READ_SIZE
            );
        }

        let content = fs::read_to_string(&abs)
            .with_context(|| format!("failed to read file: {}", abs.display()))?;

        let lines: Vec<&str> = content.lines().collect();
        let start = offset.unwrap_or(0);
        let end = limit
            .map(|l| (start + l).min(lines.len()))
            .unwrap_or(lines.len());

        if start >= lines.len() {
            return Ok(String::new());
        }

        let mut out = String::new();
        for (i, line) in lines[start..end].iter().enumerate() {
            let line_num = start + i + 1; // 1-based
            writeln!(out, "{:>6}\t{}", line_num, line)
                .expect("write to String cannot fail");
        }
        Ok(out)
    }

    // ----------------------------------------------------------------
    // write_file
    // ----------------------------------------------------------------

    /// Write content to a file. If the file already exists, a backup is
    /// created automatically before overwriting.
    pub fn write_file(&self, path: &str, content: &str) -> Result<String> {
        let abs = self.validate_path(path)?;
        debug!(?abs, "write_file");

        // Auto-backup if file exists.
        if abs.exists() {
            self.backup_path(&abs)
                .context("failed to create backup before overwrite")?;
        }

        // Ensure parent directory exists.
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        fs::write(&abs, content)
            .with_context(|| format!("failed to write file: {}", abs.display()))?;

        Ok(format!("wrote {} bytes to {}", content.len(), abs.display()))
    }

    // ----------------------------------------------------------------
    // edit_file
    // ----------------------------------------------------------------

    /// Replace an exact string in a file. Errors if the old_string is not
    /// found or appears more than once.
    pub fn edit_file(
        &self,
        path: &str,
        old_string: &str,
        new_string: &str,
    ) -> Result<String> {
        let abs = self.validate_path(path)?;
        debug!(?abs, "edit_file");

        let content = fs::read_to_string(&abs)
            .with_context(|| format!("failed to read file: {}", abs.display()))?;

        let count = content.matches(old_string).count();
        if count == 0 {
            bail!("old_string not found in {}", abs.display());
        }
        if count > 1 {
            bail!(
                "old_string found {} times in {} (must be unique)",
                count,
                abs.display()
            );
        }

        let new_content = content.replacen(old_string, new_string, 1);

        // Backup before editing.
        self.backup_path(&abs)
            .context("failed to create backup before edit")?;

        fs::write(&abs, &new_content)
            .with_context(|| format!("failed to write edited file: {}", abs.display()))?;

        Ok(format!(
            "edited {}: replaced 1 occurrence ({} bytes -> {} bytes)",
            abs.display(),
            old_string.len(),
            new_string.len()
        ))
    }

    // ----------------------------------------------------------------
    // glob
    // ----------------------------------------------------------------

    /// Find files matching a glob pattern under a base path (or workspace root).
    pub fn glob_files(
        &self,
        pattern: &str,
        base_path: Option<&str>,
    ) -> Result<Vec<String>> {
        let base = match base_path {
            Some(p) => self.validate_path(p)?,
            None => self
                .workspace_root
                .canonicalize()
                .context("failed to canonicalize workspace root")?,
        };

        // Construct the full glob pattern.
        let full_pattern = base.join(pattern);
        let pattern_str = full_pattern
            .to_str()
            .ok_or_else(|| anyhow!("invalid UTF-8 in glob pattern"))?;

        debug!(pattern = pattern_str, "glob");

        let canon_root = self.workspace_root.canonicalize()?;
        let mut results = Vec::new();

        for entry in glob::glob(pattern_str)
            .with_context(|| format!("invalid glob pattern: {}", pattern_str))?
        {
            let entry = entry.context("glob entry error")?;
            // Validate each result is within workspace.
            if let Ok(canon) = entry.canonicalize() {
                if canon.starts_with(&canon_root) {
                    results.push(canon.display().to_string());
                }
            }
        }

        results.sort();
        Ok(results)
    }

    // ----------------------------------------------------------------
    // grep
    // ----------------------------------------------------------------

    /// Search for a regex pattern in files under a path with optional context lines.
    pub fn grep(
        &self,
        pattern: &str,
        search_path: Option<&str>,
        context_lines: Option<usize>,
    ) -> Result<String> {
        let base = match search_path {
            Some(p) => self.validate_path(p)?,
            None => self
                .workspace_root
                .canonicalize()
                .context("failed to canonicalize workspace root")?,
        };

        let re = Regex::new(pattern)
            .with_context(|| format!("invalid regex pattern: {}", pattern))?;

        debug!(?base, pattern, "grep");

        let ctx = context_lines.unwrap_or(0);
        let mut output = String::new();

        if base.is_file() {
            self.grep_file(&base, &re, ctx, &mut output)?;
        } else {
            self.grep_dir(&base, &re, ctx, &mut output)?;
        }

        Ok(output)
    }

    fn grep_file(
        &self,
        path: &Path,
        re: &Regex,
        ctx: usize,
        output: &mut String,
    ) -> Result<()> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(()), // skip binary / unreadable files
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut matching_ranges: Vec<(usize, usize)> = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                let start = i.saturating_sub(ctx);
                let end = (i + ctx + 1).min(lines.len());
                matching_ranges.push((start, end));
            }
        }

        if matching_ranges.is_empty() {
            return Ok(());
        }

        // Merge overlapping ranges.
        let merged = Self::merge_ranges(&matching_ranges);

        writeln!(output, "{}:", path.display()).expect("write to String");
        for (start, end) in merged {
            for i in start..end {
                writeln!(output, "  {:>6}\t{}", i + 1, lines[i])
                    .expect("write to String");
            }
            if end < lines.len() {
                writeln!(output, "  ---").expect("write to String");
            }
        }

        Ok(())
    }

    fn grep_dir(
        &self,
        dir: &Path,
        re: &Regex,
        ctx: usize,
        output: &mut String,
    ) -> Result<()> {
        let canon_root = self.workspace_root.canonicalize()?;
        let entries = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?;

        // Sort entries for deterministic output.
        let mut sorted: Vec<_> = entries
            .filter_map(|e| e.ok())
            .collect();
        sorted.sort_by_key(|e| e.file_name());

        for entry in sorted {
            let path = entry.path();
            if let Ok(canon) = path.canonicalize() {
                if !canon.starts_with(&canon_root) {
                    continue;
                }
            }

            if path.is_dir() {
                // Skip hidden directories.
                if path
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                {
                    continue;
                }
                self.grep_dir(&path, re, ctx, output)?;
            } else if path.is_file() {
                self.grep_file(&path, re, ctx, output)?;
            }
        }

        Ok(())
    }

    fn merge_ranges(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let mut merged = vec![ranges[0]];
        for &(start, end) in &ranges[1..] {
            let last = merged.last_mut().unwrap();
            if start <= last.1 {
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }
        merged
    }

    // ----------------------------------------------------------------
    // list_directory
    // ----------------------------------------------------------------

    /// Tree-style directory listing with depth control.
    pub fn list_directory(
        &self,
        path: Option<&str>,
        depth: Option<usize>,
    ) -> Result<String> {
        let base = match path {
            Some(p) => self.validate_path(p)?,
            None => self
                .workspace_root
                .canonicalize()
                .context("failed to canonicalize workspace root")?,
        };

        let max_depth = depth.unwrap_or(2);
        debug!(?base, max_depth, "list_directory");

        let mut output = String::new();
        writeln!(output, "{}/", base.display()).expect("write to String");
        self.list_tree(&base, "", max_depth, 0, &mut output)?;
        Ok(output)
    }

    fn list_tree(
        &self,
        dir: &Path,
        prefix: &str,
        max_depth: usize,
        current_depth: usize,
        output: &mut String,
    ) -> Result<()> {
        if current_depth >= max_depth {
            return Ok(());
        }

        let canon_root = self.workspace_root.canonicalize()?;
        let entries = fs::read_dir(dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?;

        // Collect and sort entries.
        let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        sorted.sort_by_key(|e| e.file_name());

        let count = sorted.len();
        for (i, entry) in sorted.iter().enumerate() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Validate within workspace.
            if let Ok(canon) = path.canonicalize() {
                if !canon.starts_with(&canon_root) {
                    continue;
                }
            }

            let is_last = i == count - 1;
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };

            if path.is_dir() {
                writeln!(output, "{}{}{}/", prefix, connector, name)
                    .expect("write to String");
                self.list_tree(
                    &path,
                    &format!("{}{}", prefix, child_prefix),
                    max_depth,
                    current_depth + 1,
                    output,
                )?;
            } else {
                writeln!(output, "{}{}{}", prefix, connector, name)
                    .expect("write to String");
            }
        }

        Ok(())
    }

    // ----------------------------------------------------------------
    // diff
    // ----------------------------------------------------------------

    /// Simple line-by-line diff between two files.
    pub fn diff(&self, path_a: &str, path_b: &str) -> Result<String> {
        let abs_a = self.validate_path(path_a)?;
        let abs_b = self.validate_path(path_b)?;

        debug!(?abs_a, ?abs_b, "diff");

        let content_a = fs::read_to_string(&abs_a)
            .with_context(|| format!("failed to read file: {}", abs_a.display()))?;
        let content_b = fs::read_to_string(&abs_b)
            .with_context(|| format!("failed to read file: {}", abs_b.display()))?;

        let lines_a: Vec<&str> = content_a.lines().collect();
        let lines_b: Vec<&str> = content_b.lines().collect();

        let mut output = String::new();
        writeln!(output, "--- {}", abs_a.display()).expect("write to String");
        writeln!(output, "+++ {}", abs_b.display()).expect("write to String");

        // Use a simple LCS-based diff.
        let lcs = Self::lcs_table(&lines_a, &lines_b);
        let hunks = Self::compute_diff(&lines_a, &lines_b, &lcs);

        for hunk in hunks {
            writeln!(
                output,
                "@@ -{},{} +{},{} @@",
                hunk.old_start + 1,
                hunk.old_lines.len(),
                hunk.new_start + 1,
                hunk.new_lines.len()
            )
            .expect("write to String");

            for line in &hunk.old_lines {
                writeln!(output, "-{}", line).expect("write to String");
            }
            for line in &hunk.new_lines {
                writeln!(output, "+{}", line).expect("write to String");
            }
        }

        if output.lines().count() <= 2 {
            writeln!(output, "(files are identical)").expect("write to String");
        }

        Ok(output)
    }

    /// Compute an LCS table for two slices of lines.
    fn lcs_table<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Vec<usize>> {
        let m = a.len();
        let n = b.len();
        let mut table = vec![vec![0usize; n + 1]; m + 1];

        for i in 1..=m {
            for j in 1..=n {
                if a[i - 1] == b[j - 1] {
                    table[i][j] = table[i - 1][j - 1] + 1;
                } else {
                    table[i][j] = table[i - 1][j].max(table[i][j - 1]);
                }
            }
        }
        table
    }

    /// Compute diff hunks from an LCS table.
    fn compute_diff<'a>(
        a: &[&'a str],
        b: &[&'a str],
        lcs: &[Vec<usize>],
    ) -> Vec<DiffHunk<'a>> {
        let mut changes: Vec<DiffChange<'a>> = Vec::new();
        let mut i = a.len();
        let mut j = b.len();

        while i > 0 || j > 0 {
            if i > 0 && j > 0 && a[i - 1] == b[j - 1] {
                changes.push(DiffChange::Equal(a[i - 1]));
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
                changes.push(DiffChange::Add(j - 1, b[j - 1]));
                j -= 1;
            } else if i > 0 {
                changes.push(DiffChange::Remove(i - 1, a[i - 1]));
                i -= 1;
            }
        }

        changes.reverse();

        // Group consecutive non-equal changes into hunks.
        let mut hunks = Vec::new();
        let mut old_idx = 0usize;
        let mut new_idx = 0usize;

        let mut ci = 0;
        while ci < changes.len() {
            match &changes[ci] {
                DiffChange::Equal(_) => {
                    old_idx += 1;
                    new_idx += 1;
                    ci += 1;
                }
                _ => {
                    let hunk_old_start = old_idx;
                    let hunk_new_start = new_idx;
                    let mut old_lines = Vec::new();
                    let mut new_lines = Vec::new();

                    while ci < changes.len() {
                        match &changes[ci] {
                            DiffChange::Remove(_, line) => {
                                old_lines.push(*line);
                                old_idx += 1;
                                ci += 1;
                            }
                            DiffChange::Add(_, line) => {
                                new_lines.push(*line);
                                new_idx += 1;
                                ci += 1;
                            }
                            DiffChange::Equal(_) => break,
                        }
                    }

                    hunks.push(DiffHunk {
                        old_start: hunk_old_start,
                        new_start: hunk_new_start,
                        old_lines,
                        new_lines,
                    });
                }
            }
        }

        hunks
    }

    // ----------------------------------------------------------------
    // backup
    // ----------------------------------------------------------------

    /// Create a timestamped backup of a file.
    pub fn backup(&self, path: &str) -> Result<String> {
        let abs = self.validate_path(path)?;
        self.backup_path(&abs)
    }

    /// Internal backup helper that works with an already-validated path.
    fn backup_path(&self, abs_path: &Path) -> Result<String> {
        if !abs_path.exists() {
            bail!("cannot backup: file does not exist: {}", abs_path.display());
        }

        fs::create_dir_all(&self.backup_dir)
            .with_context(|| {
                format!(
                    "failed to create backup directory: {}",
                    self.backup_dir.display()
                )
            })?;

        let filename = abs_path
            .file_name()
            .ok_or_else(|| anyhow!("path has no filename"))?
            .to_string_lossy();

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}_{}", filename, timestamp);
        let backup_path = self.backup_dir.join(&backup_name);

        fs::copy(abs_path, &backup_path)
            .with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    abs_path.display(),
                    backup_path.display()
                )
            })?;

        debug!(
            src = %abs_path.display(),
            dst = %backup_path.display(),
            "backup created"
        );

        Ok(format!("backed up to {}", backup_path.display()))
    }
}

// ----------------------------------------------------------------
// Diff helper types
// ----------------------------------------------------------------

#[allow(dead_code)]
enum DiffChange<'a> {
    Equal(&'a str),
    Remove(usize, &'a str),
    Add(usize, &'a str),
}

struct DiffHunk<'a> {
    old_start: usize,
    new_start: usize,
    old_lines: Vec<&'a str>,
    new_lines: Vec<&'a str>,
}

// ================================================================
// Tests
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Create a temporary workspace for testing.
    fn setup_test_workspace() -> (tempfile::TempDir, FilesystemOps) {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let backup_dir = tmp.path().join("backups");
        let ops = FilesystemOps::new(tmp.path().to_path_buf(), backup_dir);
        (tmp, ops)
    }

    // ---- read_file tests ----

    #[test]
    fn test_read_file_normal() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("hello.txt");
        fs::write(&file, "line1\nline2\nline3\n").unwrap();

        let result = ops
            .read_file(file.to_str().unwrap(), None, None)
            .unwrap();
        assert!(result.contains("1\tline1"));
        assert!(result.contains("2\tline2"));
        assert!(result.contains("3\tline3"));
    }

    #[test]
    fn test_read_file_with_offset_limit() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("data.txt");
        fs::write(&file, "a\nb\nc\nd\ne\n").unwrap();

        let result = ops
            .read_file(file.to_str().unwrap(), Some(1), Some(2))
            .unwrap();
        // offset=1 means start at line index 1 (line 2), limit=2 -> lines 2,3
        assert!(result.contains("2\tb"));
        assert!(result.contains("3\tc"));
        assert!(!result.contains("1\ta"));
        assert!(!result.contains("4\td"));
    }

    #[test]
    fn test_read_file_not_found() {
        let (_tmp, ops) = setup_test_workspace();
        let result = ops.read_file("/nonexistent/file.txt", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_too_large() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("big.txt");
        // Create a file larger than 1 MB.
        let content = "x".repeat(MAX_READ_SIZE as usize + 1);
        fs::write(&file, &content).unwrap();

        let result = ops.read_file(file.to_str().unwrap(), None, None);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("file too large"));
    }

    // ---- write_file tests ----

    #[test]
    fn test_write_file_new() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("new.txt");

        let result = ops
            .write_file(file.to_str().unwrap(), "hello world")
            .unwrap();
        assert!(result.contains("wrote 11 bytes"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "hello world");
    }

    #[test]
    fn test_write_file_overwrite_with_backup() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("existing.txt");
        fs::write(&file, "original").unwrap();

        let result = ops
            .write_file(file.to_str().unwrap(), "new content")
            .unwrap();
        assert!(result.contains("wrote"));
        assert_eq!(fs::read_to_string(&file).unwrap(), "new content");

        // Verify backup was created.
        let backup_dir = tmp.path().join("backups");
        assert!(backup_dir.exists());
        let backups: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(backups.len(), 1);
        let backup_content = fs::read_to_string(backups[0].path()).unwrap();
        assert_eq!(backup_content, "original");
    }

    // ---- edit_file tests ----

    #[test]
    fn test_edit_file_replace() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("edit.txt");
        fs::write(&file, "hello world, hello rust").unwrap();

        let result = ops
            .edit_file(file.to_str().unwrap(), "world", "universe")
            .unwrap();
        assert!(result.contains("replaced 1 occurrence"));
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "hello universe, hello rust"
        );
    }

    #[test]
    fn test_edit_file_not_found_string() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("edit2.txt");
        fs::write(&file, "hello world").unwrap();

        let result = ops.edit_file(file.to_str().unwrap(), "xyz", "abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_edit_file_not_unique() {
        let (tmp, ops) = setup_test_workspace();
        let file = tmp.path().join("edit3.txt");
        fs::write(&file, "aaa bbb aaa").unwrap();

        let result = ops.edit_file(file.to_str().unwrap(), "aaa", "ccc");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("found 2 times"));
    }

    // ---- glob tests ----

    #[test]
    fn test_glob_find_files() {
        let (tmp, ops) = setup_test_workspace();
        fs::create_dir_all(tmp.path().join("src")).unwrap();
        fs::write(tmp.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(tmp.path().join("src/lib.rs"), "// lib").unwrap();
        fs::write(tmp.path().join("readme.md"), "# readme").unwrap();

        let results = ops.glob_files("**/*.rs", None).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|p| p.ends_with("main.rs")));
        assert!(results.iter().any(|p| p.ends_with("lib.rs")));
    }

    // ---- list_directory tests ----

    #[test]
    fn test_list_directory_with_depth() {
        let (tmp, ops) = setup_test_workspace();
        fs::create_dir_all(tmp.path().join("a/b/c")).unwrap();
        fs::write(tmp.path().join("a/file.txt"), "hi").unwrap();
        fs::write(tmp.path().join("a/b/deep.txt"), "deep").unwrap();

        // depth=1 should show 'a/' but not its children
        let result = ops.list_directory(None, Some(1)).unwrap();
        assert!(result.contains("a/"));

        // depth=2 should show contents of 'a/'
        let result2 = ops.list_directory(None, Some(2)).unwrap();
        assert!(result2.contains("a/"));
        assert!(result2.contains("file.txt"));
    }

    // ---- path validation tests ----

    #[test]
    fn test_path_traversal_rejected() {
        let (_tmp, ops) = setup_test_workspace();
        // Try to escape the workspace.
        let result = ops.read_file("../../etc/passwd", None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_absolute() {
        let (_tmp, ops) = setup_test_workspace();
        let result = ops.read_file("/etc/passwd", None, None);
        assert!(result.is_err());
    }

    // ---- MCP dispatch (integration) is tested in mcp_filesystem ----
}
