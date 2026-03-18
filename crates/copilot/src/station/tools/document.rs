use std::collections::HashMap;

use super::McpTool;

/// MCP tool server interface for document generation.
///
/// Stub implementation — `create_markdown` writes to the filesystem;
/// PDF and DOCX stubs return Ok without producing real files.
pub struct DocumentTool;

impl DocumentTool {
    pub fn new() -> Self {
        Self
    }

    /// Create a Markdown file at `path` with the given content.
    ///
    /// This is the only method with a real (minimal) implementation — it writes
    /// the content string directly to the filesystem.
    pub fn create_markdown(&self, content: &str, path: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| format!("write markdown: {e}"))
    }

    /// Create a PDF file at `path` from the given content.
    ///
    /// Stub — returns Ok without producing a real PDF.
    pub fn create_pdf(&self, _content: &str, _path: &str) -> Result<(), String> {
        Ok(())
    }

    /// Create a DOCX file at `path` from the given content.
    ///
    /// Stub — returns Ok without producing a real DOCX.
    pub fn create_docx(&self, _content: &str, _path: &str) -> Result<(), String> {
        Ok(())
    }

    /// Apply a named template with data substitutions and write the result to `path`.
    ///
    /// Stub — returns Ok without producing a real document.
    pub fn apply_template(
        &self,
        _template: &str,
        _data: &HashMap<String, String>,
        _path: &str,
    ) -> Result<(), String> {
        Ok(())
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
        "Generate documents in Markdown, PDF, and DOCX formats"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_create_markdown_writes_file() {
        let tool = DocumentTool::new();
        let dir = std::env::temp_dir().join("d1d-doc-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.md");

        tool.create_markdown("# Hello\nWorld", path.to_str().unwrap())
            .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "# Hello\nWorld");

        // Cleanup
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_pdf_and_docx_stubs_succeed() {
        let tool = DocumentTool::new();
        assert!(tool.create_pdf("content", "/tmp/stub.pdf").is_ok());
        assert!(tool.create_docx("content", "/tmp/stub.docx").is_ok());
    }

    #[test]
    fn test_apply_template_stub_succeeds() {
        let tool = DocumentTool::new();
        let mut data = HashMap::new();
        data.insert("name".to_string(), "Day1 Doctor".to_string());
        assert!(tool
            .apply_template("report", &data, "/tmp/report.md")
            .is_ok());
    }
}
