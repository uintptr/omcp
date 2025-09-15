use async_trait::async_trait;

use crate::error::Result;
use crate::types::McpParams;

#[async_trait(?Send)]
pub trait OMcpServerTrait {
    async fn listen(&mut self) -> Result<()>;
    async fn close(&mut self) -> Result<()>;
    async fn list_tools(&mut self) -> Result<String>;
    async fn call(&mut self, params: &McpParams) -> Result<String>;
}
