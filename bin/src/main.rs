use std::collections::HashMap;

use clap::Parser;
use log::{LevelFilter, info, warn};
use omcp::{
    client::{
        builder::OMcpClientBuilder,
        io::OMcpClient,
        types::{OMcpServerType, SseEvent, SseEventEndpoint},
    },
    error::{Error, Result},
    json_rpc::{JsonRPCMessage, JsonRPCMessageBuilder},
};
use rstaples::{logging::StaplesLogger, staples::printkv};
use serde_json::Value;
use tokio::{
    select,
    sync::mpsc::{self, Receiver},
};

use log::error;

const RX_BUFFER_SIZE: usize = 10;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct UserArgs {
    /// Server
    #[arg(short, long, default_value = "http://localhost:8000/mcp_server/sse")]
    server: String,

    /// Bearer Token
    #[arg(short, long)]
    bearer: Option<String>,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Debug
    #[arg(short, long)]
    debug: bool,
}

struct DumpEventHandler {
    receiver: Receiver<SseEvent>,
    endpoint: Option<SseEventEndpoint>,
}

fn build_init_message() -> Result<JsonRPCMessage> {
    let params_json = r#"{
"protocolVersion": "2025-03-26",
    "capabilities": {
      "roots": {
        "listChanged": true
      },
      "sampling": {}
    },
    "clientInfo": {
      "name": "omcpdump",
      "version": "1.0.0"
    }}
"#;

    let params: HashMap<String, Value> = serde_json::from_str(params_json)?;

    let b = JsonRPCMessageBuilder::new()
        .with_id(1)
        .with_method("initialize")
        .with_parameter(params);

    Ok(b.build())
}

impl DumpEventHandler {
    pub fn new(receiver: Receiver<SseEvent>) -> Self {
        Self {
            receiver,
            endpoint: None,
        }
    }

    async fn send_message(&self, client: &OMcpClient, msg: &JsonRPCMessage) -> Result<()> {
        match &self.endpoint {
            Some(e) => client.send(e, msg).await,
            None => Err(Error::NotConnected),
        }
    }
    async fn send_initialized(&self, client: &OMcpClient) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new()
            .with_method("notifications/initialized")
            .build();

        self.send_message(client, &msg).await
    }

    async fn send_tool_list_req(&self, client: &OMcpClient) -> Result<()> {
        let msg = JsonRPCMessageBuilder::new()
            .with_id(2)
            .with_method("tools/list")
            .build();

        self.send_message(client, &msg).await
    }

    async fn handle_jrpc(&self, client: &OMcpClient, msg: &JsonRPCMessage) -> Result<()> {
        match msg.id {
            Some(1) => {
                self.send_initialized(client).await?;
                self.send_tool_list_req(client).await
            }
            Some(2) => {
                let msg_string = serde_json::to_string_pretty(msg)?;
                println!("{msg_string}");
                Ok(())
            }
            _ => {
                dbg!(msg);
                Ok(())
            }
        }
    }

    async fn send_init_message(&self, client: &OMcpClient) -> Result<()> {
        //
        // send initialization message
        //
        let msg = build_init_message()?;
        self.send_message(client, &msg).await
    }

    async fn event_handler(&mut self, client: &OMcpClient, event: SseEvent) -> Result<()> {
        match event {
            SseEvent::Endpoint(e) => {
                self.endpoint = Some(e);
                self.send_init_message(client).await
            }
            SseEvent::JsonRpcMessage(jrpc) => self.handle_jrpc(client, &jrpc).await,
        }
    }

    pub async fn event_loop(&mut self, client: &OMcpClient) -> Result<()> {
        info!("entering event loop");

        loop {
            select! {
                ret = self.receiver.recv() => {
                    match ret{
                        Some(evt) => {
                            self.event_handler(client, evt).await?
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

fn init_logger(verbose: bool, debug: bool) -> Result<()> {
    let log = StaplesLogger::new();

    let log = match debug {
        true => log.with_log_level(LevelFilter::Debug),
        false => log,
    };

    let log = match verbose {
        true => log.with_stdout(),
        false => log,
    };

    log.start()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    init_logger(args.verbose, args.debug)?;

    println!("MCP Dumper");
    printkv("Server", &args.server);
    printkv("Verbose", args.verbose);
    printkv("Debug", args.debug);

    if let Some(b) = &args.bearer {
        // only display a little bit of the token to stdout
        if let Some(first_five) = b.get(0..5) {
            let first_five = format!("{}...", first_five);
            printkv("Bearer", first_five)
        }
    }

    let (tx, rx) = mpsc::channel::<SseEvent>(RX_BUFFER_SIZE);

    let builder = OMcpClientBuilder::new(&args.server, OMcpServerType::Sse).with_sender(tx);

    let builder = match &args.bearer {
        Some(v) => builder.with_bearer(v)?,
        None => builder,
    };

    let mut client = builder.build();

    let mut handler = DumpEventHandler::new(rx);

    client.connect().await?;

    if let Err(e) = handler.event_loop(&client).await {
        error!("{e}");
    }

    client.disconnect().await
}
