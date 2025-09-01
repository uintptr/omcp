use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::{
    client::{io::OMcpClientTrait, sse::SseClient, types::OMcpServerType},
    error::Result,
};

pub struct OMcpClientBuilder {
    pub url: String,
    pub server_type: OMcpServerType,
    pub headers: HeaderMap,
}

impl OMcpClientBuilder {
    pub fn new(server_type: OMcpServerType) -> Self {
        Self {
            url: "".into(),
            server_type,
            headers: HeaderMap::new(),
        }
    }

    pub fn with_sse_url<S>(mut self, url: S) -> Self
    where
        S: AsRef<str>,
    {
        self.url = url.as_ref().into();
        self
    }

    pub fn with_sse_bearer<S>(self, bearer: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let bearer_value = format!("Bearer {}", bearer.as_ref());
        self.with_sse_header("Authorization", bearer_value)
    }

    pub fn with_sse_header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let key = HeaderName::from_str(key.as_ref())?;
        let value = HeaderValue::from_str(value.as_ref())?;
        self.headers.insert(key, value);
        Ok(self)
    }

    pub fn build(self) -> Box<dyn OMcpClientTrait> {
        match self.server_type {
            OMcpServerType::Sse => {
                let sse = SseClient::from_builder(self);
                Box::new(sse)
            }
            OMcpServerType::Baked => {
                todo!()
            }
        }
    }
}
