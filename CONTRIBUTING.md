# Contributing to Vestige

Thank you for your interest in contributing to Vestige! This document provides guidelines and information to help you get started.

## Project Overview

Vestige is a Tauri-based desktop application combining a Rust backend with a modern web frontend. We welcome contributions of all kinds—bug fixes, features, documentation, and more.

## Development Setup

### Prerequisites

- **Rust** (stable, latest recommended): [rustup.rs](https://rustup.rs)
- **Node.js** (v18 or later): [nodejs.org](https://nodejs.org)
- **pnpm**: Install via `npm install -g pnpm`
- **Platform-specific dependencies**: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Getting Started

1. Clone the repository:
   ```bash
   git clone https://github.com/samvallad33/vestige.git
   cd vestige
   ```

2. Install frontend dependencies:
   ```bash
   pnpm install
   ```

3. Run in development mode:
   ```bash
   pnpm tauri dev
   ```

## Running Tests

```bash
# Run Rust tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

## Building

```bash
# Build Rust backend (debug)
cargo build

# Build Rust backend (release)
cargo build --release

# Build frontend
pnpm build

# Build complete Tauri application
pnpm tauri build
```

## Code Style

### Rust

We follow standard Rust conventions enforced by `rustfmt` and `clippy`.

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings
```

Please ensure your code passes both checks before submitting a PR.

### TypeScript/JavaScript

```bash
# Lint and format
pnpm lint
pnpm format
```

## Pull Request Process

1. **Fork** the repository and create a feature branch from `main`.
2. **Write tests** for new functionality.
3. **Ensure all checks pass**: `cargo fmt`, `cargo clippy`, `cargo test`.
4. **Keep commits focused**: One logical change per commit with clear messages.
5. **Update documentation** if your changes affect public APIs or behavior.
6. **Open a PR** with a clear description of what and why.

### PR Checklist

- [ ] Code compiles without warnings
- [ ] Tests pass locally
- [ ] Code is formatted (`cargo fmt`)
- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Documentation updated (if applicable)

## Issue Reporting

When reporting bugs, please include:

- **Summary**: Clear, concise description of the issue
- **Environment**: OS, Rust version (`rustc --version`), Node.js version
- **Steps to reproduce**: Minimal steps to trigger the bug
- **Expected vs actual behavior**
- **Logs/screenshots**: If applicable

For feature requests, describe the use case and proposed solution.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All contributors are expected to:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Accept constructive criticism gracefully
- Focus on what is best for the community

Harassment, discrimination, and hostile behavior will not be tolerated.

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license at your option.

---

Questions? Open a discussion or reach out to the maintainers. We're happy to help!
