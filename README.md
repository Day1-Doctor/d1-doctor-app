# Day 1 Doctor — AI-Powered System Setup Assistant

Day 1 Doctor automates the tedious, error-prone first-day developer machine setup. With one command, you get a fully configured development environment for any language or framework.

## Features

- **One-command Setup**: Install entire dev stacks with `d1-doctor install docker python node`
- **AI-Assisted Configuration**: Uses Claude to understand your setup needs and customize installations
- **Cross-Platform**: macOS, Linux, and Windows (via WSL2) support
- **Plugin Extensibility**: Build custom setup automation plugins with our SDK
- **Credit-Based**: Pay only for resources consumed, upgrade anytime
- **Offline Capable**: Local caching and offline-first design
- **Security-First**: Sandboxed command execution, clear permission boundaries

## Quick Start

### Installation

#### macOS (Homebrew)
```bash
brew install day1doctor/tap/d1-doctor
```

#### Linux
```bash
curl -fsSL https://install.day1doctor.com | sh
```

#### From Source
```bash
git clone https://github.com/day1doctor/d1-doctor-app.git
cd d1-doctor-app
cargo build --release
```

### First Run

```bash
# Start the daemon
d1-doctor daemon start

# Authenticate
d1-doctor auth login

# Install your first tool
d1-doctor install docker
```

See the [Getting Started Guide](./docs/getting-started.md) for detailed instructions.

## Project Structure

```
d1-doctor-app/
├── crates/
│   ├── daemon/       # Local daemon (Tokio async runtime)
│   ├── cli/          # Command-line client
│   ├── desktop/      # Tauri 2.0 desktop application
│   ├── sdk/          # Plugin SDK for third-party developers
│   └── common/       # Shared Rust utilities
├── docs/             # Documentation
├── examples/         # Example plugins
├── scripts/          # Build and install scripts
├── proto/            # Protocol Buffer definitions (submodule)
└── .github/          # GitHub CI/CD workflows
```

## Commands

### Daemon Management
```bash
d1-doctor daemon start        # Start the local daemon
d1-doctor daemon stop         # Stop the daemon
d1-doctor daemon restart      # Restart the daemon
d1-doctor daemon logs         # View daemon logs
```

### Setup & Installation
```bash
d1-doctor install <package>   # Install a package or setup (docker, node, python, etc.)
d1-doctor diagnose           # Scan your system for setup issues
d1-doctor status             # Show daemon and system status
```

### Authentication & Credits
```bash
d1-doctor auth login         # Authenticate with Day 1 Doctor
d1-doctor auth logout        # Clear authentication token
d1-doctor credits            # View credit balance and usage
```

### Updates
```bash
d1-doctor upgrade            # Upgrade Day 1 Doctor to latest version
```

## Architecture

Day 1 Doctor is a distributed system:

- **Cloud Orchestrator**: Receives setup requests, plans execution, manages credits
- **Local Daemon**: Runs on your machine with elevated privileges, executes commands safely
- **Clients**: CLI, desktop app, and web interface

See [Architecture Overview](./docs/architecture-overview.md) for detailed system design.

## Plugin Development

Extend Day 1 Doctor with custom plugins:

```rust
use d1_doctor_sdk::plugin_trait::Plugin;
use async_trait::async_trait;

pub struct MyPlugin;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "My custom setup plugin" }
    
    async fn execute(&self, input: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        // Your plugin logic
        Ok(serde_json::json!({"status": "success"}))
    }
}
```

See [Plugin Development Guide](./docs/plugin-development.md) and [examples/](./examples/) for complete examples.

## Documentation

- [Getting Started](./docs/getting-started.md) — Installation and first run
- [Architecture Overview](./docs/architecture-overview.md) — System design and components
- [Plugin Development](./docs/plugin-development.md) — Build custom plugins
- [API Reference](./docs/api-reference.md) — REST API and WebSocket protocol

## Building from Source

### Requirements

- Rust 1.70+ ([install](https://rustup.rs/))
- Git

### Build All Crates

```bash
./scripts/build.sh release
```

### Build Individual Crates

```bash
# Build daemon
cargo build -p d1-doctor-daemon --release

# Build CLI
cargo build -p d1-doctor-cli --release

# Build SDK
cargo build -p d1-doctor-sdk --release
```

### Run Tests

```bash
cargo test --workspace
```

### Format & Lint

```bash
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Supported Setups

Day 1 Doctor can install and configure:

- **Languages**: Node.js, Python, Go, Rust, Ruby, Java, C++
- **Databases**: PostgreSQL, MySQL, MongoDB, Redis, SQLite
- **Container Tools**: Docker, Docker Compose, Podman
- **Developer Tools**: Git, VS Code, SSH keys, GPG
- **System Utilities**: Homebrew, build essentials, package managers
- **macOS**: Xcode Command Line Tools, M1/Intel support
- **Linux**: Ubuntu, Debian, CentOS, Fedora, Arch
- **Windows**: WSL2, MSYS2, native support (coming soon)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for:

- Development setup instructions
- Code style guidelines
- Testing requirements
- Pull request process
- Issue reporting

## Security

- **Sandboxed Execution**: Commands run in isolated environment with timeout limits
- **Permission Model**: Explicit permission boundaries for plugins and operations
- **Audit Logging**: All executed commands logged locally
- **Code Review**: All changes reviewed before merge
- **Security Advisories**: See [SECURITY.md](./SECURITY.md) for vulnerability reporting

## License

Day 1 Doctor is licensed under the [MIT License](./LICENSE).

## Community

- [GitHub Issues](https://github.com/day1doctor/d1-doctor-app/issues) — Bug reports and feature requests
- [GitHub Discussions](https://github.com/day1doctor/d1-doctor-app/discussions) — Questions and ideas
- Discord — Join our community (link coming soon)
- [Plugin Registry](https://github.com/day1doctor/plugin-registry) — Discover community plugins

## FAQ

### Is Day 1 Doctor free?

Day 1 Doctor uses a credit system. You get initial credits with signup, and can purchase more as needed. The credit cost depends on the complexity of the setup task.

### Can I run Day 1 Doctor offline?

Yes! The daemon caches downloaded files and can operate offline for previously completed tasks. Cloud connectivity is required for new setups.

### Is my data secure?

Yes. The daemon runs locally on your machine with your permission. Commands are executed locally, and only setup progress is sent to the cloud. See [Privacy Policy](https://day1doctor.com/privacy) for details.

### Can I customize setups?

Absolutely! Use plugins to create custom setup automation tailored to your needs. See the [Plugin Development Guide](./docs/plugin-development.md).

### What if something goes wrong?

- Run `d1-doctor diagnose` to check system state
- View daemon logs with `d1-doctor daemon logs`
- Enable debug logging with `RUST_LOG=debug d1-doctor status`
- Report issues on [GitHub Issues](https://github.com/day1doctor/d1-doctor-app/issues)

## Roadmap

- [ ] Native Windows installer (currently WSL2 only)
- [ ] Kubernetes integration
- [ ] GPU acceleration for compilation tasks
- [ ] Offline plugin marketplace
- [ ] Team/organization management
- [ ] Docker image with pre-configured environments

## Acknowledgments

Day 1 Doctor is built with:

- [Tokio](https://tokio.rs/) — Async Rust runtime
- [Tauri](https://tauri.app/) — Desktop app framework
- [Clap](https://docs.rs/clap/) — CLI argument parsing
- [Prost](https://github.com/tokio-rs/prost) — Protocol Buffers

## Support

For help and support:

- Read the [documentation](./docs/)
- Check [existing issues](https://github.com/day1doctor/d1-doctor-app/issues)
- Start a [discussion](https://github.com/day1doctor/d1-doctor-app/discussions)
- Email support@day1doctor.com
