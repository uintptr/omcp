use crate::{
    error::{Error, Result},
    json_rpc::JsonRPCMessage,
};

use log::debug;

#[derive(Debug)]
pub enum OMcpServerType {
    Sse,
    Baked,
}

pub trait BakedMcpTool {
    fn call(&mut self, name: &str) -> Result<String>;
    fn implements(&self, name: &str) -> bool;
}

#[derive(Debug)]
pub struct SseEventEndpoint {
    pub endpoint: String,
    pub url: String,
}

impl SseEventEndpoint {
    pub fn new<S, E>(server: S, endpoint: E) -> Result<SseEventEndpoint>
    where
        S: AsRef<str>,
        E: AsRef<str>,
    {
        let root = endpoint.as_ref().split("/").nth(1).ok_or(Error::InvalidEndpoint)?;

        let root = format!("/{root}");

        let server = server.as_ref();

        let (base, _) = server.split_once(&root).ok_or(Error::InvalidEndpoint)?;

        let url = format!("{base}{}", endpoint.as_ref());

        Ok(Self {
            endpoint: endpoint.as_ref().into(),
            url,
        })
    }
}

#[derive(Debug)]
pub enum SseEvent {
    Endpoint(SseEventEndpoint),
    JsonRpcMessage(Box<JsonRPCMessage>),
}

#[derive(Default)]
pub struct SseWireEvent<'a> {
    pub server: &'a str,
    pub event: &'a str,
    pub data: &'a str,
}

impl<'a> SseWireEvent<'a> {
    pub fn new(server: &'a str) -> Self {
        Self {
            server,
            ..Default::default()
        }
    }
}

impl TryFrom<SseWireEvent<'_>> for SseEvent {
    type Error = Error;

    fn try_from(raw: SseWireEvent<'_>) -> Result<SseEvent> {
        debug!("event={} data={}", raw.event, raw.data);

        match raw.event {
            "endpoint" => {
                let endpoint = SseEventEndpoint::new(raw.server, raw.data)?;
                Ok(SseEvent::Endpoint(endpoint))
            }
            "message" => {
                let msg: JsonRPCMessage = serde_json::from_str(raw.data)?;
                Ok(SseEvent::JsonRpcMessage(Box::new(msg)))
            }
            _ => Err(Error::EventTypeNotImplemented {
                name: raw.event.to_string(),
            }),
        }
    }
}
