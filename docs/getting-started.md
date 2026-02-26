# Getting Started with Day 1 Doctor

Day 1 Doctor is an AI-powered system setup assistant that automates the installation and configuration of development tools, system utilities, and applications. This guide will get you up and running in minutes.

## Installation

### macOS (Homebrew)

The easiest way to install Day 1 Doctor on macOS:

```bash
brew install day1doctor/tap/d1-doctor
```

### Linux (curl)

For Linux systems, use our curl-based installer:

```bash
curl -fsSL https://install.day1doctor.com | sh
```

The installer will:
- Download the latest stable release
- Place the binary in `/usr/local/bin/d1-doctor`
- Start the background daemon
- Verify your installation

### Windows

Windows support is coming soon. For now, use Windows Subsystem for Linux (WSL2).

### From Source

To build from source, you'll need Rust 1.70+:

```bash
git clone https://github.com/day1doctor/d1-doctor-app.git
cd d1-doctor-app
cargo build --release
```

The compiled binary will be at `target/release/d1-doctor`.

## First Run

### 1. Start the Daemon

The daemon runs in the background and executes setup tasks:

```bash
d1-doctor daemon start
```

To verify it's running:

```bash
d1-doctor status
```

### 2. Authenticate

Authenticate with your Day 1 Doctor account using device-code flow:

```bash
d1-doctor auth login
```

This will provide you with a code to enter on https://day1doctor.com/authorize.

### 3. Run Your First Task

Install Docker with one command:

```bash
d1-doctor install docker
```

Day 1 Doctor will:
- Check system requirements
- Download necessary installers
- Execute the installation
- Verify the installation
- Display diagnostics

## Common Commands

### Check System Status

```bash
d1-doctor status
```

Shows daemon status, authentication state, available credits, and system diagnostics.

### Run Diagnostics

```bash
d1-doctor diagnose
```

Scans your system for installed software, configuration issues, and recommendations.

### View Credit Balance

```bash
d1-doctor credits
```

Displays your current credit balance and upgrade options.

### Upgrade Day 1 Doctor

```bash
d1-doctor upgrade
```

Updates to the latest version.

## Supported Setups

Day 1 Doctor can install and configure:

- **Languages**: Node.js, Python, Go, Rust, Ruby, Java
- **Databases**: PostgreSQL, MySQL, MongoDB, Redis
- **Developer Tools**: Docker, Git, VS Code extensions, SSH keys
- **macOS Development**: Xcode Command Line Tools, Homebrew
- **Linux Development**: Build essentials, package managers

## Plugins and Custom Tasks

Create custom setup automation with our Plugin SDK. See [Plugin Development Guide](./plugin-development.md) for details.

## Troubleshooting

### Daemon won't start

```bash
# Check logs
d1-doctor daemon logs

# Restart daemon
d1-doctor daemon restart
```

### Permission denied errors

Day 1 Doctor requires sudo for some operations. You may be prompted to enter your password.

### Authentication issues

Clear your authentication token and re-login:

```bash
d1-doctor auth logout
d1-doctor auth login
```

## Getting Help

- Read the full [Architecture Overview](./architecture-overview.md)
- Check the [API Reference](./api-reference.md)
- Report issues on [GitHub Issues](https://github.com/day1doctor/d1-doctor-app/issues)
- Join our community Discord

## Next Steps

- Explore the [Plugin Development Guide](./plugin-development.md) to build custom plugins
- Read [Architecture Overview](./architecture-overview.md) to understand how Day 1 Doctor works
- Check out [example plugins](../examples/) for inspiration
