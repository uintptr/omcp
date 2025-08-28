use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::sync::mpsc::Sender;

use crate::{
    client::{
        io::OMcpClient,
        sse::SseClient,
        types::{OMcpServerType, SseEvent},
    },
    error::Result,
};

pub struct OMcpClientBuilder {
    pub url: String,
    pub server_type: OMcpServerType,
    pub headers: HeaderMap,
    pub sender: Option<Sender<SseEvent>>,
}

impl OMcpClientBuilder {
    pub fn new<U>(url: U, server_type: OMcpServerType) -> Self
    where
        U: AsRef<str>,
    {
        Self {
            url: url.as_ref().into(),
            server_type,
            headers: HeaderMap::new(),
            sender: None,
        }
    }

    pub fn with_bearer<S>(self, bearer: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let bearer_value = format!("Bearer {}", bearer.as_ref());
        self.with_header("Authorization", bearer_value)
    }

    pub fn with_header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = HeaderName::from_str(key.as_ref())?;
        let value = HeaderValue::from_str(value.as_ref())?;
        self.headers.insert(key, value);
        Ok(self)
    }

    pub fn with_sender(mut self, sender: Sender<SseEvent>) -> Self {
        self.sender = Some(sender);
        self
    }

    pub fn build(self) -> OMcpClient {
        match self.server_type {
            OMcpServerType::Sse => {
                let sse = SseClient::from_builder(self);
                OMcpClient::Sse(sse)
            }
        }
    }
}
