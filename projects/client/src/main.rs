use anyhow::Result;
use clap::Parser;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParam,
    transport::TokioChildProcess,
    RoleClient,
    service::RunningService,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{io::{self, Write}};
use tokio::process::Command;
use tracing_subscriber::{self, EnvFilter};

#[derive(Parser, Debug)]
#[command(name = "calculator-chat")]
#[command(about = "A CLI chat client that can use the calculator MCP server")]
struct Args {
    #[arg(long, env, help = "Anthropic API key (can be set via ANTHROPIC_API_KEY env var or .env file)")]
    anthropic_api_key: Option<String>,
    
    #[arg(long, default_value = "../servers/calculator/target/debug/calculator")]
    calculator_path: String,
}

// Create MCP client connection to calculator server
async fn create_calculator_client(calculator_path: &str) -> Result<RunningService<RoleClient, ()>> {
    let cmd = Command::new(calculator_path);
    let transport = TokioChildProcess::new(cmd)?;
    
    let client = ()
        .serve(transport)
        .await?;
    
    // Initialize connection
    let server_info = client.peer_info();
    tracing::info!("Connected to calculator server: {server_info:#?}");
    
    // List available tools
    let tools = client.list_all_tools().await?;
    tracing::info!("Available calculator tools: {tools:#?}");
    
    Ok(client)
}

// Call a calculator tool through MCP
async fn call_calculator_tool(client: &RunningService<RoleClient, ()>, tool_name: &str, arguments: Value) -> Result<String> {
    // Convert Value to object (Map<String, Value>)
    let arguments_obj = if let Value::Object(map) = arguments {
        Some(map)
    } else {
        None
    };
    
    let tool_result = client
        .call_tool(CallToolRequestParam {
            name: tool_name.to_string().into(),
            arguments: arguments_obj,
        })
        .await?;
        
    match tool_result.content.first() {
        Some(content) => Ok(format!("{:?}", content.raw)),
        None => Ok("No result returned from tool".to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    tools: Vec<Value>,
    tool_choice: Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
}

struct ChatClient {
    http_client: reqwest::Client,
    api_key: String,
    calculator: RunningService<RoleClient, ()>,
    conversation_history: Vec<AnthropicMessage>,
}

impl ChatClient {
    fn new(api_key: String, calculator: RunningService<RoleClient, ()>) -> Self {
        let http_client = reqwest::Client::new();
        
        ChatClient {
            http_client,
            api_key,
            calculator,
            conversation_history: Vec::new(),
        }
    }

    async fn send_message(&mut self, user_message: &str) -> Result<String> {
        // Add user message to history
        self.conversation_history.push(AnthropicMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
        });

        // Get available tools from the MCP server
        let mcp_tools = self.calculator.list_all_tools().await?;
        tracing::info!("Retrieved {} tools from MCP server", mcp_tools.len());
        
        // Convert MCP tools to Anthropic tool format
        let tools: Vec<Value> = mcp_tools.iter().map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description.as_ref(),
                "input_schema": tool.input_schema
            })
        }).collect();

        // Prepare request to Anthropic API
        let request = AnthropicRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages: self.conversation_history.clone(),
            tools,
            tool_choice: json!({"type": "auto"}),
        };

        // Send request to Anthropic API
        let response = self.http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let anthropic_response: AnthropicResponse = response.json().await?;
        
        // Process response and handle tool calls
        let mut final_response = String::new();
        
        for content in &anthropic_response.content {
            match content {
                AnthropicContent::Text { text } => {
                    final_response.push_str(text);
                }
                AnthropicContent::ToolUse { id, name, input } => {
                    // Call the MCP tool
                    tracing::info!("Calling tool: {} (id: {}) with input: {:#?}", name, id, input);
                    
                    match call_calculator_tool(&self.calculator, name, input.clone()).await {
                        Ok(tool_result) => {
                            tracing::info!("Tool result: {:#?}", tool_result);
                            final_response.push_str(&format!("\n\nCalculation result: {}", tool_result));
                        }
                        Err(e) => {
                            let error_msg = format!("Error calling tool {}: {}", name, e);
                            tracing::error!("{}", error_msg);
                            final_response.push_str(&format!("\n\n{}", error_msg));
                        }
                    }
                }
            }
        }

        // Add assistant response to history
        self.conversation_history.push(AnthropicMessage {
            role: "assistant".to_string(),
            content: final_response.clone(),
        });

        Ok(final_response)
    }
}

async fn run_chat_loop(mut client: ChatClient) -> Result<()> {
    println!("🧮 Calculator Chat Client");
    println!("Ask me to perform calculations and I'll use the calculator MCP server!");
    println!("Type 'quit' or 'exit' to stop.\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input == "quit" || input == "exit" {
            println!("Goodbye!");
            break;
        }
        
        print!("🤖 Assistant: ");
        io::stdout().flush()?;
        
        match client.send_message(input).await {
            Ok(response) => {
                println!("{}\n", response);
            }
            Err(e) => {
                println!("Error: {}\n", e);
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let args = Args::parse();
    
    // Check for API key
    let api_key = args.anthropic_api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "Anthropic API key is required. Please:\n\
            1. Set ANTHROPIC_API_KEY environment variable, or\n\
            2. Add ANTHROPIC_API_KEY=your_key_here to .env file, or\n\
            3. Pass --anthropic-api-key your_key_here"
        )
    })?;
    
    tracing::info!("Starting calculator chat client");
    
    // Create calculator client connection
    let calculator = create_calculator_client(&args.calculator_path).await?;
    
    // Create chat client
    let client = ChatClient::new(api_key, calculator);
    
    run_chat_loop(client).await?;
    
    Ok(())
}
