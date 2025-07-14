use anyhow::Result;
use rmcp::{
    model::ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ServerCapabilities, ServerInfo, CallToolResult, Content, Implementation, ProtocolVersion},
    transport::stdio,
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing_subscriber::{self, EnvFilter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CalculatorError {
    DivisionByZero,
    NegativeSquareRoot { value: f64 },
    InvalidInput { message: String },
}

impl fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculatorError::DivisionByZero => write!(f, "Division by zero is not allowed"),
            CalculatorError::NegativeSquareRoot { value } => {
                write!(f, "Cannot calculate square root of negative number: {}", value)
            }
            CalculatorError::InvalidInput { message } => write!(f, "Invalid input: {}", message),
        }
    }
}

impl std::error::Error for CalculatorError {}

// Convert our custom error to McpError
impl From<CalculatorError> for McpError {
    fn from(err: CalculatorError) -> Self {
        McpError::invalid_params(err.to_string(), None)
    }
}

// Request structures for tools
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddRequest {
    #[schemars(description = "First number")]
    pub a: f64,
    #[schemars(description = "Second number")]
    pub b: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SubtractRequest {
    #[schemars(description = "Number to subtract from")]
    pub a: f64,
    #[schemars(description = "Number to subtract")]
    pub b: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MultiplyRequest {
    #[schemars(description = "First number")]
    pub a: f64,
    #[schemars(description = "Second number")]
    pub b: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DivideRequest {
    #[schemars(description = "Dividend")]
    pub a: f64,
    #[schemars(description = "Divisor")]
    pub b: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SquareRequest {
    #[schemars(description = "Number to square")]
    pub value: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SqrtRequest {
    #[schemars(description = "Number to find square root of")]
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct Calculator {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Calculator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    // Core mathematical operations with proper error handling
    fn validate_input(&self, value: f64) -> Result<(), CalculatorError> {
        if value.is_nan() {
            return Err(CalculatorError::InvalidInput {
                message: "NaN values are not allowed".to_string(),
            });
        }
        if value.is_infinite() {
            return Err(CalculatorError::InvalidInput {
                message: "Infinite values are not allowed".to_string(),
            });
        }
        Ok(())
    }

    fn validate_inputs(&self, a: f64, b: f64) -> Result<(), CalculatorError> {
        self.validate_input(a)?;
        self.validate_input(b)?;
        Ok(())
    }

    fn perform_addition(&self, a: f64, b: f64) -> Result<f64, CalculatorError> {
        self.validate_inputs(a, b)?;
        let result = a + b;
        tracing::info!("Adding {} + {} = {}", a, b, result);
        Ok(result)
    }

    fn perform_subtraction(&self, a: f64, b: f64) -> Result<f64, CalculatorError> {
        self.validate_inputs(a, b)?;
        let result = a - b;
        tracing::info!("Subtracting {} - {} = {}", a, b, result);
        Ok(result)
    }

    fn perform_multiplication(&self, a: f64, b: f64) -> Result<f64, CalculatorError> {
        self.validate_inputs(a, b)?;
        let result = a * b;
        tracing::info!("Multiplying {} * {} = {}", a, b, result);
        Ok(result)
    }

    fn perform_division(&self, a: f64, b: f64) -> Result<f64, CalculatorError> {
        self.validate_inputs(a, b)?;
        if b == 0.0 {
            tracing::warn!("Division by zero attempted: {} / {}", a, b);
            return Err(CalculatorError::DivisionByZero);
        }
        let result = a / b;
        tracing::info!("Dividing {} / {} = {}", a, b, result);
        Ok(result)
    }

    fn perform_square(&self, value: f64) -> Result<f64, CalculatorError> {
        self.validate_input(value)?;
        let result = value * value;
        tracing::info!("Squaring {} = {}", value, result);
        Ok(result)
    }

    fn perform_sqrt(&self, value: f64) -> Result<f64, CalculatorError> {
        self.validate_input(value)?;
        if value < 0.0 {
            tracing::warn!("Square root of negative number attempted: {}", value);
            return Err(CalculatorError::NegativeSquareRoot { value });
        }
        let result = value.sqrt();
        tracing::info!("Square root of {} = {}", value, result);
        Ok(result)
    }

    // Helper function to format results or errors
    fn format_result<T: std::fmt::Display>(&self, result: Result<T, CalculatorError>, operation: &str, operands: &str) -> Result<CallToolResult, McpError> {
        match result {
            Ok(value) => {
                let result_text = format!("{} = {}", operands, value);
                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            },
            Err(error) => {
                tracing::error!("Calculator error in {}: {}", operation, error);
                Err(error.into())
            }
        }
    }

    // MCP Tool interface functions
    #[tool(description = "Add two numbers together")]
    fn add(&self, Parameters(AddRequest { a, b }): Parameters<AddRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_addition(a, b);
        self.format_result(result, "addition", &format!("{} + {}", a, b))
    }

    #[tool(description = "Subtract second number from first number")]
    fn subtract(&self, Parameters(SubtractRequest { a, b }): Parameters<SubtractRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_subtraction(a, b);
        self.format_result(result, "subtraction", &format!("{} - {}", a, b))
    }

    #[tool(description = "Multiply two numbers together")]
    fn multiply(&self, Parameters(MultiplyRequest { a, b }): Parameters<MultiplyRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_multiplication(a, b);
        self.format_result(result, "multiplication", &format!("{} × {}", a, b))
    }

    #[tool(description = "Divide first number by second number")]
    fn divide(&self, Parameters(DivideRequest { a, b }): Parameters<DivideRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_division(a, b);
        self.format_result(result, "division", &format!("{} ÷ {}", a, b))
    }

    #[tool(description = "Calculate the square of a number")]
    fn square(&self, Parameters(SquareRequest { value }): Parameters<SquareRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_square(value);
        self.format_result(result, "square", &format!("{}²", value))
    }

    #[tool(description = "Calculate the square root of a number")]
    fn sqrt(&self, Parameters(SqrtRequest { value }): Parameters<SqrtRequest>) -> Result<CallToolResult, McpError> {
        let result = self.perform_sqrt(value);
        self.format_result(result, "square root", &format!("√{}", value))
    }
}

#[tool_handler]
impl ServerHandler for Calculator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("A calculator that can perform basic mathematical operations including addition, subtraction, multiplication, division, square, and square root.".to_string()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Calculator MCP server");

    let service = Calculator::new()
        .serve(stdio())
        .await?;
    
    service.waiting().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_scenarios() {
        let calc = Calculator::new();
        
        // Test division by zero
        let result = calc.perform_division(10.0, 0.0);
        assert!(matches!(result, Err(CalculatorError::DivisionByZero)));
        
        // Test negative square root
        let result = calc.perform_sqrt(-4.0);
        assert!(matches!(result, Err(CalculatorError::NegativeSquareRoot { value: -4.0 })));
        
        // Test valid operation
        let result = calc.perform_addition(5.0, 3.0);
        assert_eq!(result.unwrap(), 8.0);
    }

    #[test]
    fn test_all_operations() {
        let calc = Calculator::new();
        
        // Test addition
        assert_eq!(calc.perform_addition(10.0, 5.0).unwrap(), 15.0);
        
        // Test subtraction
        assert_eq!(calc.perform_subtraction(10.0, 3.0).unwrap(), 7.0);
        
        // Test multiplication
        assert_eq!(calc.perform_multiplication(4.0, 5.0).unwrap(), 20.0);
        
        // Test division
        assert_eq!(calc.perform_division(15.0, 3.0).unwrap(), 5.0);
        
        // Test square
        assert_eq!(calc.perform_square(4.0).unwrap(), 16.0);
        
        // Test square root
        assert_eq!(calc.perform_sqrt(9.0).unwrap(), 3.0);
    }

    #[test]
    fn test_invalid_inputs() {
        let calc = Calculator::new();
        
        // Test NaN inputs
        let result = calc.perform_addition(f64::NAN, 5.0);
        assert!(matches!(result, Err(CalculatorError::InvalidInput { .. })));
        
        // Test infinite inputs
        let result = calc.perform_multiplication(f64::INFINITY, 2.0);
        assert!(matches!(result, Err(CalculatorError::InvalidInput { .. })));
    }
}