# Rust MCP Playground

A **complete working implementation** of the Model Context Protocol (MCP) in Rust, featuring both client and server components with full tool integration.

## 🚀 Quick Start

**Run the interactive calculator chat client:**

```bash
cd projects/client
cargo run
```

This will:

- Automatically start the calculator server
- Connect to Claude via Anthropic API
- Provide a chat interface for mathematical operations

Example usage:

```txt
You: add 15 to 27
🤖 Assistant: 15 + 27 = 42

You: what's the square root of 144?
🤖 Assistant: √144 = 12
```

## 📁 Project Structure

```txt
rust-mcp-playground/
├── projects/                     # 🎯 Main implementations
│   ├── client/                   # Chat client with Anthropic integration
│   │   ├── .env                  # ANTHROPIC_API_KEY configuration
│   │   └── src/main.rs           # Full MCP client implementation
│   └── servers/calculator/       # Calculator MCP server
│       └── src/main.rs           # 6 mathematical tools
├── references/                   # 📚 Official examples
│   └── rust-mcp-sdk-examples/    # Complete SDK reference implementations
└── CLAUDE.md                     # Development guidance
```

## ✨ Features

### 🧮 Calculator Server

- **6 Mathematical Tools**: add, subtract, multiply, divide, square, sqrt
- **Error Handling**: Division by zero, negative square roots, invalid inputs
- **Input Validation**: NaN and infinity checks
- **MCP Protocol**: Latest rmcp 0.2.1 with `#[tool_router]` macros
- **Comprehensive Testing**: Unit tests for all operations

### 💬 Chat Client  

- **Natural Language Interface**: Ask questions in plain English
- **Tool Discovery**: Automatically finds and uses server capabilities
- **Anthropic Integration**: Uses Claude for intelligent responses
- **STDIO Transport**: Manages server process lifecycle
- **Environment Configuration**: API key from `.env` file

## 🛠️ Technical Implementation

**Built with the official Rust MCP SDK:**

- [`rmcp`](https://github.com/modelcontextprotocol/rust-sdk) 0.2.1 - Latest MCP implementation
- **Protocol Version**: 2024-11-05 (current MCP standard)
- **Transport**: STDIO (standard for MCP integrations)
- **Tool Registration**: Declarative macros for clean implementation

**Architecture:**

```rust
// Server tool registration
#[tool_router]
impl Calculator {
    #[tool(description = "Add two numbers together")]
    fn add(&self, Parameters(AddRequest { a, b }): Parameters<AddRequest>) 
        -> Result<CallToolResult, McpError> { /* ... */ }
}

// Client spawns server and discovers tools automatically
let transport = TokioChildProcess::new(command)?;
let client = ().serve(transport).await?;
let tools = client.list_all_tools().await?; // Discovers all 6 calculator tools
```

## 📋 Prerequisites

1. **Rust** (edition 2024)
2. **Anthropic API Key** - Add to `projects/client/.env`:

   ```txt
   ANTHROPIC_API_KEY=your_key_here
   ```

## 🧪 Development

**Run tests:**

```bash
cd projects/servers/calculator
cargo test
```

**Build components independently:**

```bash
# Server only
cd projects/servers/calculator
cargo build

# Client only  
cd projects/client
cargo build
```

**Debug server separately:**

```bash
cd projects/servers/calculator
cargo run  # Waits for STDIO input
```

## 📖 Learning Resources

- **Official MCP SDK**: <https://github.com/modelcontextprotocol/rust-sdk>
- **MCP Specification**: <https://modelcontextprotocol.io/>
- **Reference Examples**: See `references/rust-mcp-sdk-examples/` for comprehensive examples

## 🎯 What Makes This Special

This isn't just example code - it's a **fully functional MCP system** that demonstrates:

✅ **Real-world MCP integration** with working tool discovery and execution  
✅ **Production-ready patterns** with proper error handling and validation  
✅ **Modern Rust MCP development** using the latest SDK features  
✅ **Complete client-server architecture** with automatic process management  
✅ **Natural language interface** powered by Claude's intelligence  

Perfect for learning MCP concepts, building new MCP tools, or integrating MCP into existing applications!
