use std::{collections::HashMap, time::Duration};

use log::{debug, error, info, warn};

use reqwest::{Client, Response, header::HeaderMap};
use serde_json::Value;
use tokio::{
    select, spawn,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time::sleep,
};

use crate::{
    client::{
        builder::OMcpClientBuilder,
        io::EventHandlerTrait,
        types::{SseEvent, SseEventEndpoint, SseWireEvent},
    },
    error::{Error, Result},
    json_rpc::{JsonRPCInitParams, JsonRPCMessage, JsonRPCMessageBuilder},
};

#[derive(Debug)]
enum SseClienState {
    Uninitialized,
    Initialized,
    Ready,
}

#[derive(Debug)]
pub struct SseClient {
    client: Client,
    server: String,
    headers: HeaderMap,
    event_thread: Option<JoinHandle<Result<()>>>,
    quit_tx: Option<Sender<()>>,
    event_rx: Receiver<SseEvent>,
    event_tx: Option<Sender<SseEvent>>,
    endpoint: Option<SseEventEndpoint>,
    state: SseClienState,
}

const RX_BUFFER_SIZE: usize = 10;

///////////////////////////////////////////////////////////////////////////////
// Private Functions
///////////////////////////////////////////////////////////////////////////////

pub fn sse_parse_wire(server: &str, data: &[u8]) -> Result<Vec<SseEvent>> {
    let mut events: Vec<SseEvent> = Vec::new();

    let data = str::from_utf8(data)?;

    let lines: Vec<&str> = data.lines().collect();

    let mut wire = SseWireEvent::new(server);

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
            events.push(event);
            wire = SseWireEvent::new(server);
        }
    }

    Ok(events)
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

pub async fn sse_reconnect<U>(client: &Client, url: U, headers: &HeaderMap) -> Result<Response>
where
    U: AsRef<str>,
{
    loop {
        match sse_http_connect(client, &url, headers).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                error!("{e}");
                info!("reconnecting...");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn sse_parse(url: &str, sender: &Sender<SseEvent>, data: &[u8]) -> Result<()> {
    let events = sse_parse_wire(url, data)?;

    debug!("found {} events", events.len());

    for evt in events {
        if let Err(e) = sender.send(evt).await {
            error!("{e}");
            return Err(Error::EventSendFailure);
        }
    }

    Ok(())
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

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl SseClient {
    pub fn from_builder(builder: OMcpClientBuilder) -> Self {
        let (event_tx, event_rx) = mpsc::channel::<SseEvent>(RX_BUFFER_SIZE);

        SseClient {
            client: Client::new(),
            server: builder.url,
            headers: builder.headers,
            event_thread: None,
            quit_tx: None,
            event_tx: Some(event_tx),
            event_rx,
            endpoint: None,
            state: SseClienState::Uninitialized,
        }
    }

    pub async fn spawn_event_thread(&mut self) -> Result<()> {
        let (quit_tx, mut quit_rx) = mpsc::channel(1);

        // so we can reconnect
        let client = Client::new();
        let mut response = sse_http_connect(&client, &self.server, &self.headers).await?;

        let sender = self.event_tx.take().ok_or(Error::MissingSender)?;

        //
        // Now it's connected and messages from now on should be jrpcs
        //
        let server = self.server.clone();
        let headers = self.headers.clone();

        let event_thread = spawn(async move {
            let mut connected = true;

            loop {
                tokio::select! {
                    _ = quit_rx.recv() => {
                        info!("quit requested");
                        break
                    }
                    Ok(new_connection) = sse_reconnect(&client, &server, &headers), if !connected => {
                        //stream = response.bytes_stream();
                        response = new_connection;
                        connected = true;
                    }
                    item = response.chunk(), if connected => {
                        match item{
                            Ok(Some(chunk)) => sse_parse(&server, &sender, &chunk).await?,
                            Ok(None) => {} // nothing to read?
                            Err(e) => {
                                error!("{e}");
                                connected = false;
                            }
                        }
                    }
                }
            }
            Ok(())
        });

        self.event_thread = Some(event_thread);
        self.quit_tx = Some(quit_tx);

        //
        // Make sure it's ready to accept commands from
        // the user
        //
        self.initialize_loop().await?;

        //
        // Wait until we're initialized
        //
        Ok(())
    }

    pub async fn join_event_thread(&mut self) -> Result<()> {
        match self.quit_tx.take() {
            Some(tx) => match tx.send(()).await {
                Ok(_) => match self.event_thread.take() {
                    Some(v) => v.await?,
                    None => Ok(()),
                },
                Err(e) => {
                    error!("{e}");
                    Err(Error::QuitSignalFailure)
                }
            },
            None => Ok(()),
        }
    }

    pub async fn send_message(&self, msg: &JsonRPCMessage) -> Result<()> {
        match &self.endpoint {
            Some(endpoint) => {
                //
                // we have to use a different http connection for this one
                //
                let json_msg = serde_json::to_string_pretty(msg)?;

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

    async fn send_hello(&self) -> Result<()> {
        let msg = build_init_message()?;
        self.send_message(&msg).await
    }

    async fn send_initialized(&self) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new().with_method("notifications/initialized").build();
        self.send_message(&msg).await
    }

    async fn send_list_tools(&self) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new().with_id(2).with_method("tools/list").build();
        self.send_message(&msg).await
    }

    async fn initialize_loop(&mut self) -> Result<()> {
        if let SseClienState::Ready = self.state {
            return Ok(());
        }

        loop {
            select! {
                ret = self.event_rx.recv() => {
                    match ret{
                        Some(event) => {
                            match event{
                                SseEvent::Endpoint(e) => {
                                    self.endpoint = Some(e);
                                    self.state = SseClienState::Uninitialized;
                                    self.send_hello().await?;
                                    self.state = SseClienState::Initialized;
                                }
                                SseEvent::JsonRpcMessage(_msg) => {

                                    match self.state {
                                        SseClienState::Uninitialized => {
                                            break Err(Error::ConnectionStateFailure);
                                        }
                                        SseClienState::Initialized => {
                                            self.send_initialized().await?;
                                            self.state = SseClienState::Ready;
                                            break Ok(())
                                        }
                                        SseClienState::Ready => {
                                            break Err(Error::ConnectionStateFailure);
                                        }
                                    }
                                }
                            }
                        }
                        None => {
                            warn!("empty message");
                            break Err(Error::ConnectionStateFailure);
                        }
                    }
                }
            }
        }
    }

    pub async fn list_tools(&mut self) -> Result<JsonRPCMessage> {
        self.send_list_tools().await?;

        match self.event_rx.recv().await {
            Some(v) => match v {
                SseEvent::JsonRpcMessage(msg) => Ok(*msg),
                _ => Err(Error::NotFound),
            },
            None => Err(Error::ConnectionFailure),
        }
    }

    pub async fn call_tool(&mut self) -> Result<JsonRPCMessage> {
        unimplemented!()
    }

    pub async fn event_loop<H>(&mut self, user_handler: H) -> Result<()>
    where
        H: EventHandlerTrait,
    {
        loop {
            self.initialize_loop().await?;

            select! {
                ret = self.event_rx.recv() => {
                    match ret{
                        Some(event) => {
                            match event{
                                //
                                // In case the socket dies and the whole thing
                                // needs to re-initialize
                                //
                                SseEvent::Endpoint(e) => {
                                    self.endpoint = Some(e);
                                    self.state = SseClienState::Uninitialized;
                                    self.send_hello().await?;
                                    self.state = SseClienState::Initialized;
                                    //
                                    // So we go back into the initialize_loop
                                    //
                                    break;
                                }
                                SseEvent::JsonRpcMessage(msg) => {
                                    user_handler.event_handler(&msg).await?
                                }
                            }
                        }
                        None => {
                            warn!("empty message");
                            break;
                        }
                    }
                }

            }
        }

        Ok(())
    }
}
