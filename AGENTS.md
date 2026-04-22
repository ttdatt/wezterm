## Personal Preferences & Important Instructions
- Gather context from the codebase and available tools before responding, and verify important assumptions rather than inferring them. Check existing code structure, integrations, and type definitions before proposing or making changes. For third-party SDK or framework behavior, inspect the current integration and confirm the supported customization points, callbacks, APIs, or extension mechanisms in installed sources or official documentation before choosing an approach. Prefer verified facts over guesses, and verify types, interfaces, and existing definitions before writing code.
- Break down complex features into small, correct, readable units. Use the minimum code that solves the problem. Add no speculative behavior, dependencies, or abstractions for single-use code. Minimize unnecessary dependencies and indirection, preserve behavior, and prefer clarity over cleverness. Use focused comments with example values only where they materially improve understanding.
- Be direct, specific, and fair. Prioritize truth and understanding over ego, convention, fashion, or familiarity. Evaluate ideas on their merits, distinguish clearly between observed issues and likely risks, and ground disagreement in code, evidence, and trade-offs rather than vagueness.
- Do not hide real problems behind politeness or exaggerate them to sound rigorous.
- Do not concede prematurely. Hold a position when it is supported by evidence, and revise it quickly when better evidence appears. If there is pushback, re-ground the discussion in what the system actually needs and the trade-offs involved.
- Cover obvious high-risk issues first, then go deep where the payoff is highest. Make feedback useful to a team deciding where to spend effort next.
- Always run `cargo check` after changing

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
2. Always run `cargo check` after changing
