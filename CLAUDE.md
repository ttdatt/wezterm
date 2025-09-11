# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WezTerm is a GPU-accelerated cross-platform terminal emulator and multiplexer written in Rust. It provides modern terminal features with GPU acceleration, multiplexing capabilities, and extensive configuration options through Lua scripting.

## Architecture

### Core Components

- **`term/`** - Core terminal model implementation, agnostic of windowing systems. Contains terminal escape sequence processing, terminal state management, and core emulation logic. This is where terminal behavior compatibility with xterm is implemented.

- **`wezterm/`** - Main GUI application entry point and coordination layer.

- **`wezterm-gui/`** - GUI renderer for the terminal model. Contains windowing system integration, OpenGL/GPU rendering, input handling, and platform-specific GUI code.

- **`wezterm-font/`** - Font discovery, loading, and rendering system. Handles fontconfig integration, font fallback chains, and text shaping.

- **`mux/`** - Terminal multiplexer implementation. Manages multiple terminal sessions, client-server architecture, and session persistence.

- **`config/`** - Configuration system with Lua scripting support for extensive customization.

- **`termwiz/`** - Lower-level terminal utilities and abstractions used by the core terminal implementation.

### Key Libraries and Utilities

- **`vtparse/`** - ANSI/VT escape sequence parser
- **`wezterm-ssh/`** - SSH client implementation for remote terminal sessions  
- **`window/`** - Cross-platform windowing abstractions
- **`pty/`** (portable-pty) - Cross-platform pseudo-terminal implementation
- **`bidi/`** - Bidirectional text support for RTL languages

### Lua API Crates

The `lua-api-crates/` directory contains Rust crates that expose functionality to the Lua configuration system:
- `mux/` - Multiplexer API
- `window-funcs/` - Window management functions
- `spawn-funcs/` - Process spawning utilities
- `filesystem/` - File system operations
- And many other utility modules

## Development Commands

### Building
```bash
# Build all main components
make build
# Or build specific packages
cargo build -p wezterm
cargo build -p wezterm-gui
cargo build -p wezterm-mux-server
```

### Testing
```bash
# Run all tests
make test
# Or use cargo nextest (preferred test runner)
cargo nextest run
# Test specific no_std crates
cargo nextest run -p wezterm-escape-parser
```

### Code Quality
```bash
# Type checking (fast iteration)
cargo check
# Format code (requires nightly)
cargo +nightly fmt
# Or use make target
make fmt
```

### Documentation
```bash
# Build documentation site
make docs
# Serve docs locally with auto-reload
make servedocs
# Or directly
ci/build-docs.sh serve
```

### Dependencies
```bash
# Install system dependencies for your platform
./get-deps
# Include test dependencies
./get-deps --testing
# Include documentation dependencies  
./get-deps --docs
```

## Project Structure

This is a Rust workspace with multiple related crates. The workspace is defined in the root `Cargo.toml` with member crates for different components. Each major component (`wezterm`, `wezterm-gui`, `term`, etc.) is its own crate with its own `Cargo.toml`.

### Key Directories

- `ci/` - Continuous integration scripts and build automation
- `docs/` - Documentation source files (MkDocs format)
- `assets/` - Icons, images, and other static assets
- `deps/` - Vendored dependencies (Cairo, fontconfig, etc.)
- `lua-api-crates/` - Lua API bindings for configuration system
- `test-data/` - Test fixtures and reference data

## Configuration and Scripting

WezTerm uses Lua for configuration, providing a powerful scripting interface. The configuration system allows users to customize nearly every aspect of terminal behavior, appearance, and functionality through Lua scripts.

## Platform Support

WezTerm targets multiple platforms with platform-specific code:
- **Linux** - X11, Wayland support
- **macOS** - Native Cocoa/Metal integration  
- **Windows** - Win32 API integration
- **FreeBSD** - Unix-like system support

The codebase includes extensive cross-platform abstractions to handle differences between windowing systems, font rendering, and system integration.

## Testing Philosophy

- Write tests for terminal behavior using the test helpers in `term/src/test/`
- Include clear comments explaining test intent
- Focus testing on core logic in helpers and terminal behavior
- Use `cargo nextest` as the preferred test runner for better performance and output

## Contributing Guidelines

When making changes:
1. Ensure terminal compatibility follows xterm behavior where possible
2. Run `cargo test --all` and `cargo fmt --all` before submitting
3. Include tests for new functionality, especially terminal escape sequences
4. Add documentation for user-facing features
5. Consider cross-platform implications for windowing and system integration changes