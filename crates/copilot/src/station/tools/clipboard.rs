use std::sync::Mutex;

use super::McpTool;

/// MCP tool server interface for clipboard access.
///
/// Stub implementation — uses an in-memory buffer instead of the system clipboard.
pub struct ClipboardTool {
    buffer: Mutex<String>,
}

impl ClipboardTool {
    pub fn new() -> Self {
        Self {
            buffer: Mutex::new(String::new()),
        }
    }

    /// Read the current clipboard contents.
    ///
    /// Stub — reads from an internal buffer.
    pub fn read(&self) -> String {
        self.buffer.lock().unwrap().clone()
    }

    /// Write content to the clipboard.
    ///
    /// Stub — writes to an internal buffer.
    pub fn write(&self, content: &str) {
        *self.buffer.lock().unwrap() = content.to_string();
    }
}

impl Default for ClipboardTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for ClipboardTool {
    fn name(&self) -> &str {
        "clipboard"
    }

    fn risk_level(&self) -> &str {
        "low"
    }

    fn description(&self) -> &str {
        "Read and write to the system clipboard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_read_write() {
        let tool = ClipboardTool::new();

        // Initially empty.
        assert_eq!(tool.read(), "");

        tool.write("hello clipboard");
        assert_eq!(tool.read(), "hello clipboard");
    }

    #[test]
    fn test_clipboard_overwrite() {
        let tool = ClipboardTool::new();

        tool.write("first");
        assert_eq!(tool.read(), "first");

        tool.write("second");
        assert_eq!(tool.read(), "second");
    }
}
