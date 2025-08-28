use std::time::Duration;

use log::{debug, error, info};

//use futures_util::StreamExt;

use reqwest::{Client, Response, header::HeaderMap};
use tokio::{
    spawn,
    sync::mpsc::{self, Sender},
    task::JoinHandle,
    time::sleep,
};

use crate::{
    client::{
        builder::OMcpClientBuilder,
        types::{SseEvent, SseEventEndpoint, SseWireEvent},
    },
    error::{Error, Result},
    json_rpc::JsonRPCMessage,
};

#[derive(Debug)]
pub struct SseClient {
    client: Client,
    url: String,
    headers: HeaderMap,
    event_thread: Option<JoinHandle<Result<()>>>,
    quit_tx: Option<Sender<()>>,
    sender: Option<Sender<SseEvent>>,
}

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

    let response = client
        .get(url.as_ref())
        .headers(headers_clone)
        .send()
        .await?;

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

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl SseClient {
    pub fn from_builder(builder: OMcpClientBuilder) -> Self {
        SseClient {
            client: Client::new(),
            url: builder.url,
            headers: builder.headers,
            event_thread: None,
            quit_tx: None,
            sender: builder.sender,
        }
    }

    pub async fn spawn_event_thread(&mut self) -> Result<()> {
        let (tx, mut quit_rx) = mpsc::channel(1);

        // so we can reconnect
        let client = Client::new();
        let mut response = sse_http_connect(&client, &self.url, &self.headers).await?;

        let sender = self.sender.take().ok_or(Error::MissingSender)?;

        //
        // Now it's connected and messages from now on should be jrpcs
        //

        let url = self.url.clone();
        let headers = self.headers.clone();

        let event_thread = spawn(async move {
            let mut connected = true;

            loop {
                tokio::select! {
                    _ = quit_rx.recv() => {
                        info!("quit requested");
                        break
                    }
                    Ok(new_connection) = sse_reconnect(&client, &url, &headers), if !connected => {
                        //stream = response.bytes_stream();
                        response = new_connection;
                        connected = true;
                    }
                    item = response.chunk(), if connected => {
                        match item{
                            Ok(Some(chunk)) => sse_parse(&url, &sender, &chunk).await?,
                            Ok(None) => {} // nothing to read?
                            Err(e) => {
                                error!("{e}");
                                //connected = false;
                            }
                        }
                    }
                }
            }
            Ok(())
        });
        self.event_thread = Some(event_thread);
        self.quit_tx = Some(tx);

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

    pub async fn send_message(
        &self,
        endpoint: &SseEventEndpoint,
        msg: &JsonRPCMessage,
    ) -> Result<()> {
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
}
