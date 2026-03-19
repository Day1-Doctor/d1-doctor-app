//! E2E Tool Dispatch Tests — routing, unknown tools, workspace sandboxing, HTML extraction.
//! All tests use real tool implementations with no actual LLM or network calls.

use serde_json::json;

use d1_copilot_lib::station::executor::ToolDispatcher;
use d1_copilot_lib::station::tools::filesystem::FilesystemTool;
use d1_copilot_lib::station::tools::web_fetch::WebFetchTool;

// ---------------------------------------------------------------------------
// Test: ToolDispatcher routes filesystem.read correctly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_routes_read_file() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-read");
    std::fs::create_dir_all(&dir).ok();
    let dispatcher = ToolDispatcher::new(dir.clone());

    let file_path = dir.join("test_read.txt");
    std::fs::write(&file_path, "hello from tools test").unwrap();

    let result = dispatcher
        .execute("read_file", json!({"path": file_path.to_str().unwrap()}))
        .await;
    assert!(result.success, "read_file should succeed");
    assert_eq!(result.output["content"], "hello from tools test");

    // Cleanup.
    std::fs::remove_file(&file_path).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher routes filesystem.read via alias
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_routes_filesystem_read_alias() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-alias");
    std::fs::create_dir_all(&dir).ok();
    let dispatcher = ToolDispatcher::new(dir.clone());

    let file_path = dir.join("alias_test.txt");
    std::fs::write(&file_path, "alias content").unwrap();

    // Use the "filesystem.read" alias.
    let result = dispatcher
        .execute("filesystem.read", json!({"path": file_path.to_str().unwrap()}))
        .await;
    assert!(result.success, "filesystem.read alias should work");
    assert_eq!(result.output["content"], "alias content");

    // Cleanup.
    std::fs::remove_file(&file_path).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher handles unknown tool gracefully
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_unknown_tool_graceful_error() {
    let dispatcher = ToolDispatcher::with_cwd();
    let result = dispatcher.execute("nonexistent_tool", json!({})).await;

    assert!(!result.success, "unknown tool should fail");
    let error_msg = result.output["error"].as_str().unwrap();
    assert!(
        error_msg.contains("unknown tool"),
        "error should mention 'unknown tool', got: {}",
        error_msg
    );
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher handles missing required parameter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_missing_required_param() {
    let dispatcher = ToolDispatcher::with_cwd();
    let result = dispatcher.execute("read_file", json!({})).await;

    assert!(!result.success);
    let error_msg = result.output["error"].as_str().unwrap();
    assert!(error_msg.contains("missing required parameter"));
}

// ---------------------------------------------------------------------------
// Test: filesystem tools respect workspace sandbox
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_filesystem_sandbox_rejects_escape() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-sandbox");
    std::fs::create_dir_all(&dir).ok();
    let tool = FilesystemTool::new(dir.clone());

    // Try to read a file outside the workspace via path traversal.
    let result = tool.read("/etc/passwd");
    assert!(result.is_err(), "reading /etc/passwd should be rejected");
    let err = result.unwrap_err();
    assert!(
        err.contains("outside the workspace") || err.contains("cannot resolve"),
        "error should mention workspace boundary: {}",
        err
    );

    // Try relative path traversal.
    let traversal = format!("{}/../../../etc/hosts", dir.display());
    let result = tool.read(&traversal);
    assert!(result.is_err(), "path traversal should be rejected");

    // Cleanup.
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: filesystem write + read roundtrip within sandbox
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_filesystem_write_read_roundtrip() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-rw");
    std::fs::create_dir_all(&dir).ok();
    let dispatcher = ToolDispatcher::new(dir.clone());

    let file_path = dir.join("roundtrip.txt");
    let path_str = file_path.to_str().unwrap();

    // Write.
    let write_result = dispatcher
        .execute("write_file", json!({"path": path_str, "content": "roundtrip data"}))
        .await;
    assert!(write_result.success, "write should succeed");

    // Read back.
    let read_result = dispatcher
        .execute("read_file", json!({"path": path_str}))
        .await;
    assert!(read_result.success, "read should succeed");
    assert_eq!(read_result.output["content"], "roundtrip data");

    // Cleanup.
    std::fs::remove_file(&file_path).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: web_fetch extract_text strips HTML correctly
// ---------------------------------------------------------------------------

#[test]
fn tools_extract_text_strips_html() {
    let html = "<html><body><h1>Title</h1><p>Paragraph text</p></body></html>";
    let text = WebFetchTool::extract_text(html);

    assert!(text.contains("Title"), "should contain heading text");
    assert!(text.contains("Paragraph text"), "should contain paragraph text");
    assert!(!text.contains("<h1>"), "should not contain HTML tags");
    assert!(!text.contains("<p>"), "should not contain HTML tags");
    assert!(!text.contains("</body>"), "should not contain closing tags");
}

// ---------------------------------------------------------------------------
// Test: extract_text strips script and style tags
// ---------------------------------------------------------------------------

#[test]
fn tools_extract_text_strips_script_and_style() {
    let html = r#"
        <html>
        <head><style>body { color: red; }</style></head>
        <body>
            <script>var x = "dangerous";</script>
            <p>Safe content only</p>
        </body>
        </html>
    "#;
    let text = WebFetchTool::extract_text(html);

    assert!(text.contains("Safe content only"));
    assert!(!text.contains("color: red"), "style content should be stripped");
    assert!(!text.contains("dangerous"), "script content should be stripped");
}

// ---------------------------------------------------------------------------
// Test: extract_text decodes HTML entities
// ---------------------------------------------------------------------------

#[test]
fn tools_extract_text_decodes_entities() {
    let html = "<p>A &amp; B &lt; C &gt; D &quot;E&quot; &#39;F&#39;</p>";
    let text = WebFetchTool::extract_text(html);
    assert!(text.contains("A & B < C > D \"E\" 'F'"));
}

// ---------------------------------------------------------------------------
// Test: extract_text handles empty input
// ---------------------------------------------------------------------------

#[test]
fn tools_extract_text_empty_input() {
    assert_eq!(WebFetchTool::extract_text(""), "");
}

// ---------------------------------------------------------------------------
// Test: extract_text passes through plain text
// ---------------------------------------------------------------------------

#[test]
fn tools_extract_text_plain_text_passthrough() {
    let text = WebFetchTool::extract_text("Just plain text without any HTML");
    assert_eq!(text, "Just plain text without any HTML");
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher glob finds files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_glob_finds_files() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-glob");
    std::fs::create_dir_all(&dir).ok();
    std::fs::write(dir.join("alpha.txt"), "a").ok();
    std::fs::write(dir.join("beta.txt"), "b").ok();
    std::fs::write(dir.join("gamma.rs"), "c").ok();

    let dispatcher = ToolDispatcher::new(dir.clone());
    let result = dispatcher.execute("glob", json!({"pattern": "*.txt"})).await;
    assert!(result.success);

    let matches = result.output["matches"].as_array().unwrap();
    assert!(matches.len() >= 2, "should find at least 2 .txt files");

    // Cleanup.
    std::fs::remove_file(dir.join("alpha.txt")).ok();
    std::fs::remove_file(dir.join("beta.txt")).ok();
    std::fs::remove_file(dir.join("gamma.rs")).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher list_dir works correctly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_list_dir() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-listdir");
    std::fs::create_dir_all(&dir).ok();
    std::fs::write(dir.join("file1.txt"), "").ok();
    std::fs::write(dir.join("file2.txt"), "").ok();

    let dispatcher = ToolDispatcher::new(dir.clone());
    let result = dispatcher
        .execute("list_dir", json!({"path": dir.to_str().unwrap()}))
        .await;
    assert!(result.success);

    let entries = result.output["entries"].as_array().unwrap();
    assert!(entries.iter().any(|e| e.as_str() == Some("file1.txt")));
    assert!(entries.iter().any(|e| e.as_str() == Some("file2.txt")));

    // Cleanup.
    std::fs::remove_file(dir.join("file1.txt")).ok();
    std::fs::remove_file(dir.join("file2.txt")).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher create_markdown produces correct file
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_create_markdown() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-md");
    std::fs::create_dir_all(&dir).ok();
    let dispatcher = ToolDispatcher::new(dir.clone());

    let file_path = dir.join("output.md");
    let path_str = file_path.to_str().unwrap();

    let result = dispatcher
        .execute("create_markdown", json!({"content": "# Report\n\nContent here", "path": path_str}))
        .await;
    assert!(result.success);
    assert_eq!(result.output["path"].as_str().unwrap(), path_str);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "# Report\n\nContent here");

    // Cleanup.
    std::fs::remove_file(&file_path).ok();
    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: ToolDispatcher tracks execution duration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tools_dispatcher_tracks_duration() {
    let dispatcher = ToolDispatcher::with_cwd();
    let result = dispatcher
        .execute("read_file", json!({"path": "/nonexistent/path"}))
        .await;
    // Even for errors, duration should be tracked (and reasonable).
    assert!(result.duration_ms < 10_000, "duration should be reasonable");
}

// ---------------------------------------------------------------------------
// Test: supported_tools list is complete
// ---------------------------------------------------------------------------

#[test]
fn tools_supported_tools_list_complete() {
    let dir = std::env::temp_dir().join("d1d-e2e-tools-list");
    std::fs::create_dir_all(&dir).ok();
    let dispatcher = ToolDispatcher::new(dir.clone());

    let tools = dispatcher.supported_tools();
    let expected = vec![
        "read_file", "write_file", "glob", "list_dir",
        "web_search", "fetch_url", "create_markdown",
    ];
    for tool in &expected {
        assert!(tools.contains(tool), "missing tool: {}", tool);
    }

    std::fs::remove_dir(&dir).ok();
}

// ---------------------------------------------------------------------------
// Test: tool definitions have correct OpenAI function format
// ---------------------------------------------------------------------------

#[test]
fn tools_definitions_valid_openai_format() {
    let defs = ToolDispatcher::tool_definitions();
    assert!(!defs.is_empty());

    for def in &defs {
        assert_eq!(def["type"], "function", "type should be 'function'");
        assert!(def["function"]["name"].is_string(), "name should be string");
        assert!(def["function"]["description"].is_string(), "description should be string");
        assert!(def["function"]["parameters"].is_object(), "parameters should be object");
        assert_eq!(
            def["function"]["parameters"]["type"], "object",
            "parameters type should be 'object'"
        );
    }
}
