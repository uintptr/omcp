use async_trait::async_trait;

use crate::{
    client::{baked::BackedClient, sse::SseClient},
    error::{Error, Result},
    json_rpc::JsonRPCMessage,
    types::{McpParams, McpTool},
};

pub enum OMcpClient {
    Sse(SseClient),
    Baked(BackedClient),
}
#[async_trait]
pub trait EventHandlerTrait {
    async fn event_handler(&self, msg: &JsonRPCMessage) -> Result<()>;
}

impl OMcpClient {
    pub async fn connect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.spawn_event_thread().await,
            OMcpClient::Baked(_baked) => Ok(()),
        }
    }

    pub async fn send(&self, msg: &JsonRPCMessage) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.send_message(msg).await,
            OMcpClient::Baked(baked) => baked.send_message(msg),
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.join_event_thread().await,
            OMcpClient::Baked(_baked) => Ok(()),
        }
    }

    pub async fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        match self {
            OMcpClient::Sse(sse) => sse.list_tools().await,
            OMcpClient::Baked(baked) => baked.list_tools(),
        }
    }

    pub async fn call<P>(&mut self, mcp_params: P) -> Result<String>
    where
        P: AsRef<McpParams>,
    {
        match self {
            OMcpClient::Sse(sse) => sse.call(mcp_params).await,
            OMcpClient::Baked(baked) => baked.call_tool(&mcp_params.as_ref().tool_name),
        }
    }

    pub async fn event_loop<H>(&mut self, handler: H) -> Result<()>
    where
        H: EventHandlerTrait,
    {
        match self {
            OMcpClient::Sse(sse) => sse.event_loop(handler).await,
            OMcpClient::Baked(_baked) => Err(Error::NotImplemented),
        }
    }
}
