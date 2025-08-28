use crate::{
    client::{sse::SseClient, types::SseEventEndpoint},
    error::Result,
    json_rpc::JsonRPCMessage,
};

#[derive(Debug)]
pub enum OMcpClient {
    Sse(SseClient),
}

impl OMcpClient {
    pub async fn connect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.spawn_event_thread().await,
        }
    }

    pub async fn send(&self, endpoint: &SseEventEndpoint, msg: &JsonRPCMessage) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.send_message(endpoint, msg).await,
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        match self {
            OMcpClient::Sse(sse) => sse.join_event_thread().await,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    // Import everything from the parent module

    use crate::client::{builder::OMcpClientBuilder, types::OMcpServerType};

    #[tokio::test]
    async fn builder() {
        let _ = OMcpClientBuilder::new("http://localhost:1234", OMcpServerType::Sse)
            .with_header("k", "v")
            .unwrap()
            .with_header("k2", "v2")
            .unwrap()
            .build();
    }
}
