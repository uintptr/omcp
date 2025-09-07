use std::{collections::HashMap, sync::atomic::AtomicU64};

use async_trait::async_trait;
use log::{debug, error};

use reqwest::{Client, Response, header::HeaderMap};
use serde_json::Value;

use crate::{
    client::{
        builder::OMcpClientBuilder,
        io::OMcpClientTrait,
        types::{SseEvent, SseEventEndpoint, SseWireEvent},
    },
    error::{Error, Result},
    json_rpc::{JsonRPCInitParams, JsonRPCMessage, JsonRPCMessageBuilder, JsonRPCParameters},
    types::{McpParams, McpTool},
};

#[derive(Debug)]
enum SseClientState {
    Uninitialized,
    Initialized,
    Ready,
}

#[derive(Debug)]
pub struct SseClient {
    client: Client,
    server: String,
    headers: HeaderMap,
    endpoint: Option<SseEventEndpoint>,
    state: SseClientState,
    msg_id: AtomicU64,
    response: Option<Response>,
}

///////////////////////////////////////////////////////////////////////////////
// Private Functions
///////////////////////////////////////////////////////////////////////////////

pub fn sse_parse_wire<S, D>(server: S, data: D) -> Result<SseEvent>
where
    S: AsRef<str>,
    D: AsRef<[u8]>,
{
    let data = str::from_utf8(data.as_ref())?;

    let lines: Vec<&str> = data.lines().collect();

    let mut wire = SseWireEvent::new(server.as_ref());

    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.starts_with(": ") {
            debug!("ignoring {line}");
            continue;
        }

        if let Some(event) = line.strip_prefix("event: ") {
            debug!("{event}");
            wire.event = event;
        } else if let Some(data) = line.strip_prefix("data: ") {
            debug!("{data}");
            wire.data = data;
        }

        if !wire.data.is_empty() && !wire.event.is_empty() {
            let event: SseEvent = wire.try_into()?;
            return Ok(event);
        }
    }

    Err(Error::NotFound)
}

async fn sse_http_connect<U>(client: &Client, url: U, headers: &HeaderMap) -> Result<Response>
where
    U: AsRef<str>,
{
    let headers_clone = headers.clone();

    let response = client.get(url.as_ref()).headers(headers_clone).send().await?;

    match response.status().is_success() {
        true => Ok(response),
        false => Err(Error::ConnectionFailure),
    }
}

fn build_init_message() -> Result<JsonRPCMessage> {
    let init_params = JsonRPCInitParams::new();
    let init_string = serde_json::to_string(&init_params)?;

    let params: HashMap<String, Value> = serde_json::from_str(&init_string)?;

    let b = JsonRPCMessageBuilder::new()
        .with_id(1)
        .with_method("initialize")
        .with_parameter(params);

    Ok(b.build())
}

async fn read_all(response: &mut Response) -> Result<Vec<u8>> {
    let mut vec: Vec<u8> = Vec::new();

    loop {
        match response.chunk().await {
            Ok(v) => match v {
                Some(data) => {
                    let data = data.to_vec();
                    vec.extend(data);

                    if vec.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }
                None => break,
            },
            Err(e) => {
                error!("{e}");
                return Err(e.into());
            }
        }
    }

    Ok(vec)
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl SseClient {
    pub fn from_builder(builder: OMcpClientBuilder) -> Self {
        SseClient {
            client: Client::new(),
            server: builder.url,
            headers: builder.headers,
            endpoint: None,
            state: SseClientState::Uninitialized,
            msg_id: AtomicU64::new(1),
            response: None,
        }
    }

    pub async fn recv_message(&mut self) -> Result<JsonRPCMessage> {
        let data = match self.response.as_mut() {
            Some(v) => read_all(v).await?,
            None => return Err(Error::NotConnected),
        };

        match self.sse_parse(data)? {
            SseEvent::Endpoint(_e) => Err(Error::NotConnected),
            SseEvent::JsonRpcMessage(msg) => Ok(*msg),
        }
    }
    pub async fn send_message<M>(&self, msg: M) -> Result<()>
    where
        M: AsRef<JsonRPCMessage>,
    {
        match &self.endpoint {
            Some(endpoint) => {
                //
                // we have to use a different http connection for this one
                //
                let json_msg = serde_json::to_string_pretty(msg.as_ref())?;

                debug!("sending: {json_msg}");

                let headers = self.headers.clone();

                self.client
                    .post(&endpoint.url)
                    .header("Content-Type", "application/json")
                    .headers(headers)
                    .body(json_msg)
                    .send()
                    .await?;

                Ok(())
            }
            None => Err(Error::NotConnected),
        }
    }

    fn sse_parse<D>(&mut self, data: D) -> Result<SseEvent>
    where
        D: AsRef<[u8]>,
    {
        sse_parse_wire(&self.server, data)
    }

    //
    // This'll also handle reconnections
    //
    async fn init_connection(&mut self) -> Result<()> {
        loop {
            //
            // server sends a hello message first
            //
            let data = match self.response.as_mut() {
                Some(v) => read_all(v).await?,
                None => break Err(Error::NotConnected),
            };

            let event = self.sse_parse(data)?;

            match event {
                SseEvent::Endpoint(e) => {
                    self.state = SseClientState::Uninitialized;
                    self.endpoint = Some(e);
                }
                SseEvent::JsonRpcMessage(_msg) => {}
            }

            //
            // we have a msg
            //
            match self.state {
                SseClientState::Uninitialized => {
                    self.state = SseClientState::Initialized;
                    self.send_hello().await?;
                }
                SseClientState::Initialized => {
                    self.send_initialized().await?;
                    self.state = SseClientState::Ready;
                    break Ok(());
                }
                SseClientState::Ready => {
                    break Ok(());
                }
            }
        }
    }

    async fn send_hello(&self) -> Result<()> {
        let msg = build_init_message()?;
        self.send_message(msg).await
    }

    async fn send_initialized(&self) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new().with_method("notifications/initialized").build();
        self.send_message(msg).await
    }

    async fn send_list_tools(&self) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new().with_id(2).with_method("tools/list").build();
        self.send_message(msg).await
    }
}

#[async_trait(?Send)]
impl OMcpClientTrait for SseClient {
    async fn connect(&mut self) -> Result<()> {
        let response = sse_http_connect(&self.client, &self.server, &self.headers).await?;
        self.response = Some(response);

        self.init_connection().await?;

        Ok(())
    }
    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
    async fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        self.send_list_tools().await?;

        let msg = self.recv_message().await?;

        let mut results = msg.result.ok_or(Error::NotFound)?;

        let tool_value = results.remove("tools").ok_or(Error::NotFound)?;

        dbg!(&tool_value);

        let tools: Vec<McpTool> = match serde_json::from_value(tool_value) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                return Err(e.into());
            }
        };

        Ok(tools)
    }
    async fn call(&mut self, mcp_params: &McpParams) -> Result<String> {
        let id = self.msg_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let params: JsonRPCParameters = mcp_params.as_ref().try_into()?;

        let msg = JsonRPCMessageBuilder::new()
            .with_id(id)
            .with_method("tools/call")
            .with_parameter(params)
            .build();

        if let Err(e) = self.send_message(msg).await {
            error!("{e}");
            return Ok(format!("Error: {e}"));
        }

        match self.recv_message().await {
            Ok(v) => {
                let results = v.result.ok_or(Error::NotFound)?;
                let results = serde_json::to_string_pretty(&results)?;
                Ok(results)
            }
            Err(e) => Ok(format!("Error: {e}")),
        }
    }
}
