use async_trait::async_trait;

use crate::{
    error::Result,
    types::{McpParams, McpTool},
};

#[async_trait(?Send)]
pub trait OMcpClientTrait {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn list_tools(&mut self) -> Result<Vec<McpTool>>;
    async fn call(&mut self, mcp_params: &McpParams) -> Result<String>;
}
