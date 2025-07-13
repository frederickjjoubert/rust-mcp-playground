# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust MCP (Model Context Protocol) playground for experimenting with MCP implementations. The repository contains both client and server examples using the Rust MCP SDK from `https://github.com/modelcontextprotocol/rust-sdk`.

## Project Structure

- **`projects/`** - Main development area for custom MCP implementations
  - `client/` - CLI client implementation (currently basic "Hello, world!")
  - `servers/calculator/` - Calculator MCP server implementation (currently basic)
- **`references/rust-mcp-sdk-examples/`** - Complete working examples from the official Rust MCP SDK
  - `clients/` - Various client examples (SSE, stdio, HTTP, OAuth, etc.)
  - `servers/` - Various server examples (counter, memory, auth, etc.)
  - `transport/` - Transport layer examples (TCP, WebSocket, Unix socket, etc.)
  - `simple-chat-client/` - Complete chat client implementation
  - `rig-integration/` - Integration with Rig framework
- **`notes/`** - Documentation and research notes about MCP

## Common Development Commands

### Building and Running Examples

Build reference server examples:
```bash
cd references/rust-mcp-sdk-examples
cargo build --release --example servers_counter_stdio
```

Run reference examples:
```bash
cd references/rust-mcp-sdk-examples
cargo run -p mcp-server-examples --example servers_counter_stdio
cargo run -p mcp-client-examples --example clients_everything_stdio
```

### Building Project Components

Build the custom client:
```bash
cd projects/client
cargo build
cargo run
```

Build the custom calculator server:
```bash
cd projects/servers/calculator
cargo build  
cargo run
```

### Testing with MCP Inspector

Use the official MCP inspector to test servers:
```bash
npx @modelcontextprotocol/inspector
```

### Claude Desktop Integration

To integrate with Claude Desktop, add server configuration to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "counter": {
      "command": "PATH-TO/rust-mcp-playground/references/rust-mcp-sdk-examples/target/release/examples/servers_counter_stdio",
      "args": []
    }
  }
}
```

## Architecture

### MCP Protocol Implementation

The project uses the `rmcp` crate (Rust MCP SDK) which provides:
- **Transport layers**: stdio, SSE, HTTP, WebSocket, TCP, Unix sockets
- **Client implementations**: Various transport-specific clients
- **Server implementations**: Tool and resource serving capabilities
- **Authentication**: OAuth and other auth mechanisms

### Key Dependencies

Reference examples use:
- `rmcp` - Core MCP SDK with various transport features
- `tokio` - Async runtime
- `serde`/`serde_json` - Serialization
- `tracing` - Logging and observability
- `anyhow` - Error handling
- `axum` - HTTP server framework
- `reqwest` - HTTP client

### Transport Patterns

1. **stdio** - Standard input/output (most common for Claude Desktop)
2. **SSE** - Server-Sent Events for web integration
3. **HTTP** - RESTful and streaming HTTP
4. **WebSocket** - Bidirectional communication
5. **TCP/Unix sockets** - Direct socket communication

## Development Notes

- The `projects/` directory contains minimal placeholder implementations
- The `references/` directory contains comprehensive working examples
- All examples use Rust edition 2024
- Server examples typically implement counter, calculator, or memory tools
- Client examples demonstrate various connection and interaction patterns
- Use `tracing::info!` for logging rather than `println!`
- Follow the async/await patterns established in the reference examples