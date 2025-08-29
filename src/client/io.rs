use async_trait::async_trait;

use crate::{client::sse::SseClient, error::Result, json_rpc::JsonRPCMessage};

#[derive(Debug)]
pub enum OMcpClient {
    Sse(SseClient),
}
#[async_trait]
pub trait EventHandlerTrait {
    async fn event_handler(&self, msg: &JsonRPCMessage) -> Result<()>;
}

impl OMcpClient {
    pub async fn connect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.spawn_event_thread().await,
        }
    }

    pub async fn send(&self, msg: &JsonRPCMessage) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.send_message(msg).await,
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.join_event_thread().await,
        }
    }

    pub async fn event_loop<H>(&mut self, handler: H) -> Result<()>
    where
        H: EventHandlerTrait,
    {
        match self {
            OMcpClient::Sse(sse) => sse.event_loop(handler).await,
        }
    }
}
