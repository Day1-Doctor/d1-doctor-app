# Plugin Development Guide

Day 1 Doctor plugins extend the platform with custom setup automation tasks. This guide covers plugin creation, testing, and distribution.

## What is a Plugin?

A plugin is a Rust library that implements the `Plugin` trait from the Day 1 Doctor SDK. Plugins can:

- Execute shell commands with sandboxing
- Query system state and installed software
- Register MCP tools for AI agent use
- Store persistent state in the local database
- Request user input via the TUI

## Plugin Anatomy

A minimal plugin implements three lifecycle methods:

```rust
use d1_doctor_sdk::plugin_trait::Plugin;
use async_trait::async_trait;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }
    
    fn version(&self) -> &str {
        "0.1.0"
    }
    
    fn description(&self) -> &str {
        "My custom setup plugin"
    }
    
    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Plugin logic here
        Ok(serde_json::json!({ "status": "success" }))
    }
}
```

## Project Structure

Create a new plugin with Cargo:

```bash
cargo new --lib my-d1-plugin
cd my-d1-plugin
```

Update `Cargo.toml`:

```toml
[package]
name = "my-d1-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
d1-doctor-sdk = "0.1"
d1-doctor-common = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
anyhow = "1"
async-trait = "0.1"
```

## Building MCP Tools

Plugins register MCP (Model Context Protocol) tools that AI agents can invoke:

```rust
use d1_doctor_sdk::mcp_tools;

#[async_trait]
impl Plugin for MyPlugin {
    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Register a tool for the AI agent
        let tool_schema = serde_json::json!({
            "name": "install_package",
            "description": "Install a system package",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "package": { "type": "string" }
                }
            }
        });
        
        mcp_tools::register_tool("install_package", tool_schema);
        
        Ok(serde_json::json!({ "tools_registered": 1 }))
    }
}
```

## Accessing System State

Query installed software and system info:

```rust
use d1_doctor_common::config::AppConfig;

async fn check_dependencies(&self) -> anyhow::Result<()> {
    // Query system state
    let config = AppConfig::default();
    println!("Data directory: {}", config.data_dir);
    
    // Execute commands safely
    let output = std::process::Command::new("which")
        .arg("node")
        .output()?;
    
    if output.status.success() {
        println!("Node.js is installed");
    }
    
    Ok(())
}
```

## Testing Plugins

The SDK provides a test harness:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use d1_doctor_sdk::testing;
    
    #[test]
    fn test_plugin() {
        let plugin = MyPlugin;
        testing::test_plugin(&plugin);
    }
}
```

Run tests:

```bash
cargo test
```

## Publishing Plugins

### Step 1: Add Plugin Metadata

Update `Cargo.toml`:

```toml
[package]
name = "my-d1-plugin"
version = "0.1.0"
edition = "2021"
description = "My custom setup plugin for Day 1 Doctor"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-org/my-d1-plugin"
keywords = ["d1-doctor", "plugin", "setup"]

[lib]
crate-type = ["cdylib"]  # Build as dynamic library
```

### Step 2: Publish to crates.io

```bash
cargo publish
```

### Step 3: Register with Day 1 Doctor

Submit a PR to the [plugin registry](https://github.com/day1doctor/plugin-registry) with:

- Plugin name
- crates.io URL
- Description
- Supported platforms (Linux, macOS, Windows)
- Permission requirements

## Plugin Permissions

Plugins run in a sandboxed environment with explicit permission model:

- **shell**: Execute shell commands
- **filesystem**: Read/write files
- **network**: Make HTTP requests
- **database**: Store state in local DB
- **system_info**: Query hardware and software
- **install**: Install packages (requires sudo)

Declare permissions in plugin manifest:

```toml
[package.metadata.d1-doctor]
permissions = ["shell", "filesystem", "system_info"]
```

## Example Plugins

See the [examples/](../examples/) directory for complete plugin examples:

- `hello-world-plugin/`: Minimal plugin template
- `git-setup-plugin/`: Configure Git with SSH keys

## Debugging

Enable debug logging in plugins:

```rust
use tracing::{debug, info};

debug!("Plugin state: {:?}", self.state);
info!("Starting setup task");
```

Run with verbose logging:

```bash
RUST_LOG=debug d1-doctor status
```

## Best Practices

1. **Idempotency**: Plugins should be safe to run multiple times
2. **Error Handling**: Use `anyhow::Result` for comprehensive error messages
3. **User Feedback**: Report progress via structured JSON output
4. **Validation**: Check system requirements before executing tasks
5. **Documentation**: Include README with usage examples

## Support

- Check existing plugins in the [registry](https://github.com/day1doctor/plugin-registry)
- Ask questions on [GitHub Discussions](https://github.com/day1doctor/d1-doctor-app/discussions)
- Report bugs on [GitHub Issues](https://github.com/day1doctor/d1-doctor-app/issues)
