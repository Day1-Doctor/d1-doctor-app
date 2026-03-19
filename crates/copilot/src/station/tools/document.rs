use std::collections::HashMap;
use std::path::Path;

use super::McpTool;

/// MCP tool server interface for document generation.
///
/// Supports creating Markdown and plain text files with path validation.
/// PDF and DOCX generation are not yet implemented and return clear error
/// messages indicating the limitation.
pub struct DocumentTool;

impl DocumentTool {
    pub fn new() -> Self {
        Self
    }

    /// Validate that the path has the expected extension and that its parent
    /// directory exists (creating it if necessary).
    fn validate_and_prepare(path: &str, expected_ext: &str) -> Result<(), String> {
        let p = Path::new(path);

        // Validate extension.
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !ext.eq_ignore_ascii_case(expected_ext) {
            return Err(format!(
                "expected '.{expected_ext}' extension, got '.{ext}'"
            ));
        }

        // Ensure parent directory exists.
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create directories for '{}': {e}", parent.display()))?;
            }
        }

        Ok(())
    }

    /// Create a Markdown file at `path` with the given content.
    ///
    /// Validates that the path ends with `.md` and creates parent directories
    /// if they don't exist.
    pub fn create_markdown(&self, content: &str, path: &str) -> Result<String, String> {
        Self::validate_and_prepare(path, "md")?;
        std::fs::write(path, content)
            .map_err(|e| format!("write markdown '{}': {e}", path))?;
        Ok(path.to_string())
    }

    /// Create a plain text file at `path` with the given content.
    ///
    /// Validates that the path ends with `.txt` and creates parent directories
    /// if they don't exist.
    pub fn create_text(&self, content: &str, path: &str) -> Result<String, String> {
        Self::validate_and_prepare(path, "txt")?;
        std::fs::write(path, content)
            .map_err(|e| format!("write text '{}': {e}", path))?;
        Ok(path.to_string())
    }

    /// Create a PDF file at `path` from the given content.
    ///
    /// Not yet implemented — returns a clear error message.
    pub fn create_pdf(&self, _content: &str, _path: &str) -> Result<String, String> {
        Err(
            "PDF generation is not yet implemented. \
             Use create_markdown() and convert with an external tool."
                .to_string(),
        )
    }

    /// Create a DOCX file at `path` from the given content.
    ///
    /// Not yet implemented — returns a clear error message.
    pub fn create_docx(&self, _content: &str, _path: &str) -> Result<String, String> {
        Err(
            "DOCX generation is not yet implemented. \
             Use create_markdown() and convert with an external tool."
                .to_string(),
        )
    }

    /// Apply a named template with data substitutions and write the result to `path`.
    ///
    /// Performs simple `{{key}}` placeholder substitution on the template content
    /// and writes the result as a Markdown file.
    pub fn apply_template(
        &self,
        template: &str,
        data: &HashMap<String, String>,
        path: &str,
    ) -> Result<String, String> {
        let mut output = template.to_string();
        for (key, value) in data {
            let placeholder = format!("{{{{{key}}}}}");
            output = output.replace(&placeholder, value);
        }
        self.create_markdown(&output, path)
    }
}

impl Default for DocumentTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for DocumentTool {
    fn name(&self) -> &str {
        "document"
    }

    fn risk_level(&self) -> &str {
        "medium"
    }

    fn description(&self) -> &str {
        "Generate documents in Markdown and text formats"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_markdown_writes_file() {
        let tool = DocumentTool::new();
        let dir = std::env::temp_dir().join("d1d-doc-test-md");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.md");

        let result = tool
            .create_markdown("# Hello\nWorld", path.to_str().unwrap())
            .unwrap();

        assert_eq!(result, path.to_str().unwrap());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "# Hello\nWorld");

        // Cleanup.
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_markdown_wrong_extension() {
        let tool = DocumentTool::new();
        let result = tool.create_markdown("content", "/tmp/test.txt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected '.md'"));
    }

    #[test]
    fn test_create_text_writes_file() {
        let tool = DocumentTool::new();
        let dir = std::env::temp_dir().join("d1d-doc-test-txt");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("notes.txt");

        let result = tool
            .create_text("Some plain text", path.to_str().unwrap())
            .unwrap();

        assert_eq!(result, path.to_str().unwrap());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "Some plain text");

        // Cleanup.
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_create_text_wrong_extension() {
        let tool = DocumentTool::new();
        let result = tool.create_text("content", "/tmp/test.md");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expected '.txt'"));
    }

    #[test]
    fn test_create_markdown_creates_parent_dirs() {
        let dir = std::env::temp_dir().join("d1d-doc-nested");
        let path = dir.join("a").join("b").join("deep.md");
        // Ensure it doesn't exist yet.
        std::fs::remove_dir_all(&dir).ok();

        let tool = DocumentTool::new();
        tool.create_markdown("# Deep", path.to_str().unwrap())
            .unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "# Deep");

        // Cleanup.
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_pdf_and_docx_return_errors() {
        let tool = DocumentTool::new();
        assert!(tool.create_pdf("content", "/tmp/stub.pdf").is_err());
        assert!(tool.create_docx("content", "/tmp/stub.docx").is_err());

        let pdf_err = tool.create_pdf("x", "/tmp/x.pdf").unwrap_err();
        assert!(pdf_err.contains("not yet implemented"));

        let docx_err = tool.create_docx("x", "/tmp/x.docx").unwrap_err();
        assert!(docx_err.contains("not yet implemented"));
    }

    #[test]
    fn test_apply_template() {
        let tool = DocumentTool::new();
        let dir = std::env::temp_dir().join("d1d-doc-test-tmpl");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("report.md");

        let template = "# Report for {{name}}\n\nStatus: {{status}}";
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Day1 Doctor".to_string());
        data.insert("status".to_string(), "Shipped".to_string());

        tool.apply_template(template, &data, path.to_str().unwrap())
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# Report for Day1 Doctor"));
        assert!(content.contains("Status: Shipped"));
        assert!(!content.contains("{{"));

        // Cleanup.
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_mcp_trait() {
        let tool = DocumentTool::new();
        assert_eq!(tool.name(), "document");
        assert_eq!(tool.risk_level(), "medium");
        assert!(!tool.description().is_empty());
    }
}
