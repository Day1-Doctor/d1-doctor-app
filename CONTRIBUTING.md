# Contributing to Day 1 Doctor

Thank you for your interest in contributing to Day 1 Doctor! This document provides guidelines and instructions for contributing.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please review and follow our Code of Conduct:

- Be respectful and inclusive
- Welcome diverse perspectives
- Focus on what is best for the community
- Show empathy towards others

## Getting Started

### Prerequisites

- Rust 1.70+ ([install](https://rustup.rs/))
- Git
- macOS, Linux, or Windows (WSL2)

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/d1-doctor-app.git
   cd d1-doctor-app
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/day1doctor/d1-doctor-app.git
   ```

4. Create a development branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. Build and test:
   ```bash
   ./scripts/build.sh        # Debug build
   ./scripts/build.sh release # Release build
   ```

## Development Workflow

### Making Changes

1. Create a feature branch from `main`:
   ```bash
   git checkout upstream/main
   git checkout -b feature/description
   ```

2. Make your changes following the code style (see below)

3. Write or update tests:
   ```bash
   cargo test --workspace
   ```

4. Commit your changes with descriptive messages:
   ```bash
   git commit -m "Description of changes"
   ```

5. Push to your fork:
   ```bash
   git push origin feature/description
   ```

6. Open a Pull Request on GitHub

### Code Style

We follow Rust conventions and enforce style with:

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Lint with Clippy
cargo clippy --workspace --all-targets -- -D warnings
```

All PRs must pass these checks before merging.

### Testing

- Write tests for new functionality
- Ensure all tests pass: `cargo test --workspace`
- Aim for >80% code coverage
- Test across platforms (Linux, macOS) when possible

### Documentation

- Update relevant documentation for new features
- Add doc comments to public APIs
- Update README.md if user-facing changes
- Include examples for complex functionality

## Project Structure

```
d1-doctor-app/
├── crates/
│   ├── daemon/       # Local daemon service
│   ├── cli/          # Command-line interface
│   ├── desktop/      # GUI application (Tauri)
│   ├── sdk/          # Plugin SDK
│   └── common/       # Shared code
├── docs/             # User documentation
├── examples/         # Example plugins
├── scripts/          # Build scripts
└── proto/            # Protocol Buffer definitions
```

## Contribution Types

### Bug Fixes

1. Check existing issues for duplicates
2. Create an issue if one doesn't exist
3. Include reproduction steps
4. Fix the bug with tests
5. Reference the issue in your PR

### Features

1. Discuss major features in an issue first
2. Get approval from maintainers
3. Implement with comprehensive tests
4. Update documentation
5. Add examples if appropriate

### Documentation

- Fix typos and unclear sections
- Add examples and clarifications
- Improve API documentation
- Update architecture diagrams

### Plugin Examples

- Create example plugins in `examples/`
- Demonstrate best practices
- Include README with usage

## Commit Messages

Use clear, descriptive commit messages:

```
Short summary (50 chars or less)

More detailed explanation if needed. Wrap at 72 characters.
Explain what the change does and why, not how.

- Bullet points for multiple changes
- Each on separate lines

Fixes #123
Closes #456
```

## Pull Request Process

1. **Update from upstream**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure all checks pass**:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

3. **Write a descriptive PR description**:
   - What does this PR do?
   - Why is this change needed?
   - How was it tested?
   - Any breaking changes?

4. **Address review feedback**:
   - Make requested changes
   - Respond to all comments
   - Push updates to your branch
   - Re-request review

5. **Rebase before merge**:
   ```bash
   git rebase upstream/main
   git push origin feature/description --force-with-lease
   ```

## Release Process

### Version Numbering

We use [Semantic Versioning](https://semver.org/):
- MAJOR: Breaking API changes
- MINOR: New features, backwards compatible
- PATCH: Bug fixes, no API changes

### Creating a Release

1. Update version in `Cargo.toml` files
2. Update CHANGELOG.md
3. Create a git tag: `git tag vX.Y.Z`
4. Push tag: `git push upstream vX.Y.Z`
5. GitHub Actions handles the release

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/day1doctor/d1-doctor-app/discussions)
- **Bugs**: Create an [Issue](https://github.com/day1doctor/d1-doctor-app/issues)
- **Discussions**: Join our Discord (link in README)
- **Email**: dev@day1doctor.com

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Clap Documentation](https://docs.rs/clap/)
- [Protocol Buffers Guide](https://protobuf.dev/)

## Areas We Need Help With

- Cross-platform testing (especially Windows/WSL)
- Plugin examples and templates
- Documentation improvements
- Performance optimizations
- macOS/Linux native features

## Recognition

All contributors are recognized in:
- CONTRIBUTORS.md
- Release notes
- GitHub contributors page

Thank you for contributing to Day 1 Doctor!
