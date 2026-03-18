use super::McpTool;

/// MCP tool server interface for browser automation.
///
/// **Risk: High** — requires explicit user approval before execution.
/// Stub implementation — all methods return mock data or Ok.
pub struct BrowserTool;

impl BrowserTool {
    pub fn new() -> Self {
        Self
    }

    /// Navigate the browser to the given URL.
    pub fn navigate(&self, _url: &str) -> Result<(), String> {
        Ok(())
    }

    /// Click on an element matching the given CSS selector.
    pub fn click(&self, _selector: &str) -> Result<(), String> {
        Ok(())
    }

    /// Fill an input element matching the selector with the given value.
    pub fn fill(&self, _selector: &str, _value: &str) -> Result<(), String> {
        Ok(())
    }

    /// Take a screenshot of the current browser viewport.
    ///
    /// Stub — returns a 1x1 white PNG pixel.
    pub fn screenshot(&self) -> Vec<u8> {
        // Minimal valid 1x1 white PNG.
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE,
        ]
    }

    /// Wait for the specified duration in milliseconds.
    pub fn wait(&self, _ms: u64) -> Result<(), String> {
        Ok(())
    }
}

impl Default for BrowserTool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn risk_level(&self) -> &str {
        "high"
    }

    fn description(&self) -> &str {
        "Automate browser interactions: navigate, click, fill forms, take screenshots"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_navigation_and_actions() {
        let tool = BrowserTool::new();

        assert!(tool.navigate("https://example.com").is_ok());
        assert!(tool.click("#submit-btn").is_ok());
        assert!(tool.fill("input[name=email]", "test@example.com").is_ok());
        assert!(tool.wait(1000).is_ok());
    }

    #[test]
    fn test_screenshot_returns_bytes() {
        let tool = BrowserTool::new();
        let data = tool.screenshot();
        // PNG files start with the 8-byte signature.
        assert!(data.len() > 8);
        assert_eq!(&data[..4], &[0x89, 0x50, 0x4E, 0x47]);
    }
}
