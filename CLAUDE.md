# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust MCP (Model Context Protocol) playground for experimenting with MCP implementations. The repository contains both working client and server examples using the latest Rust MCP SDK from `https://github.com/modelcontextprotocol/rust-sdk`.

## Project Structure

- **`projects/`** - Main development area for custom MCP implementations
  - `client/` - **Full-featured chat client** that connects to MCP servers with Anthropic API integration
  - `servers/calculator/` - **Complete calculator MCP server** with 6 mathematical operations
- **`references/rust-mcp-sdk-examples/`** - Complete working examples from the official Rust MCP SDK
  - `clients/` - Various client examples (SSE, stdio, HTTP, OAuth, etc.)
  - `servers/` - Various server examples (counter, memory, auth, etc.)
  - `transport/` - Transport layer examples (TCP, WebSocket, Unix socket, etc.)
  - `simple-chat-client/` - Complete chat client implementation
  - `rig-integration/` - Integration with Rig framework
- **`notes/`** - Documentation and research notes about MCP

## Common Development Commands

### Running the Full MCP System

**Primary Usage**: Run the chat client (it automatically starts the calculator server):

```bash
cd projects/client
cargo run
```

The client will:

- Automatically spawn the calculator server process via STDIO transport
- Connect to the Anthropic API using the API key from `.env` file
- Provide an interactive chat interface for mathematical operations

### Building Individual Components

Build and test the calculator server independently:

```bash
cd projects/servers/calculator
cargo build  
cargo run  # Runs server waiting for STDIO input
```

Build the client:

```bash
cd projects/client
cargo build
```

### Running Tests

Test the calculator server functionality:

```bash
cd projects/servers/calculator
cargo test
```

### Testing with MCP Inspector

Use the official MCP inspector to test servers:

```bash
npx @modelcontextprotocol/inspector
```

## Architecture

### MCP Protocol Implementation

The project uses the `rmcp` crate (Rust MCP SDK) which provides:

- **Transport layers**: stdio, SSE, HTTP, WebSocket, TCP, Unix sockets
- **Client implementations**: Various transport-specific clients
- **Server implementations**: Tool and resource serving capabilities
- **Authentication**: OAuth and other auth mechanisms

### Current Implementation Status

**Calculator Server** (`projects/servers/calculator/`):

- ✅ **6 Working Tools**: add, subtract, multiply, divide, square, sqrt
- ✅ **Full MCP Integration**: Using `rmcp` 0.2.1 with `#[tool_router]` and `#[tool_handler]` macros
- ✅ **Error Handling**: Division by zero, negative square roots, invalid inputs
- ✅ **Input Validation**: NaN and infinity checks
- ✅ **Comprehensive Tests**: Unit tests for all operations and error scenarios
- ✅ **Clean Logging**: Structured logging with tracing

**Chat Client** (`projects/client/`):

- ✅ **Anthropic API Integration**: Uses Claude for natural language to tool calls
- ✅ **MCP Tool Discovery**: Automatically discovers and uses server tools  
- ✅ **STDIO Transport**: Spawns and manages calculator server process
- ✅ **Environment Configuration**: API key loaded from `.env` file
- ✅ **Interactive Chat**: User-friendly command-line interface

### Key Dependencies

Both projects use:

- `rmcp = "0.2.1"` - Latest official Rust MCP SDK with full tool support
- `tokio` - Async runtime for MCP protocol handling
- `serde`/`serde_json` - JSON serialization for MCP messages
- `tracing` - Structured logging and observability  
- `anyhow` - Error handling across the application
- `schemars` - JSON Schema generation for tool parameters

**Client-specific**:

- `reqwest` - HTTP client for Anthropic API calls
- `clap` - Command-line argument parsing
- `dotenvy` - Environment variable loading from `.env`

**Server-specific**:

- `rmcp` tool macros - `#[tool_router]`, `#[tool_handler]`, `#[tool]` for declarative tool registration

### Transport Patterns

1. **stdio** - Standard input/output (most common for Claude Desktop)
2. **SSE** - Server-Sent Events for web integration
3. **HTTP** - RESTful and streaming HTTP
4. **WebSocket** - Bidirectional communication
5. **TCP/Unix sockets** - Direct socket communication

## Development Notes

### Current State

- The `projects/` directory contains **fully functional MCP implementations**
- The `references/` directory contains comprehensive working examples from the official SDK
- All code uses Rust edition 2024 and the latest MCP protocol (2024-11-05)

### MCP Architecture Patterns

**Tool Registration Pattern** (used in calculator server):

```rust
#[tool_router]
impl Calculator {
    #[tool(description = "Add two numbers together")]
    fn add(&self, Parameters(AddRequest { a, b }): Parameters<AddRequest>) -> Result<CallToolResult, McpError> {
        // Implementation
    }
}

#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo { /* ... */ }
}
```

**Client-Server Communication** (STDIO transport):

- Client spawns server process via `TokioChildProcess::new(command)`
- Communication flows through stdin/stdout pipes
- MCP protocol handles tool discovery and execution automatically

### Best Practices Established

- **Error Handling**: Use custom error types that convert to `McpError`
- **Logging**: Use `tracing::info!` for server operations, not `println!`
- **Validation**: Validate all inputs (NaN, infinity, domain-specific constraints)
- **Tool Schemas**: Use `schemars::JsonSchema` for automatic parameter documentation
- **Testing**: Include comprehensive unit tests for all business logic
- **Environment**: Use `.env` files for sensitive configuration like API keys

### File Organization

```txt
projects/
├── client/
│   ├── .env                 # ANTHROPIC_API_KEY configuration
│   ├── Cargo.toml          # rmcp 0.2.1 with client features
│   └── src/main.rs         # Chat client with Anthropic integration
└── servers/calculator/
    ├── Cargo.toml          # rmcp 0.2.1 with server features  
    └── src/main.rs         # Calculator server with 6 tools
```
