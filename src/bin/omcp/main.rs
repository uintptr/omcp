use async_trait::async_trait;
use clap::{Parser, Subcommand};
use log::{
    LevelFilter, {error, info},
};
use omcp::{
    client::{
        builder::OMcpClientBuilder,
        io::{EventHandlerTrait, OMcpClient},
        types::OMcpServerType,
    },
    error::{Error, Result},
    json_rpc::JsonRPCMessageBuilder,
};

use omcp::json_rpc::JsonRPCMessage;

use rstaples::{logging::StaplesLogger, staples::printkv};

#[derive(Parser)]
struct UserArgsDump {
    /// Server
    #[arg(short, long)]
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

#[derive(Subcommand)]
enum Commands {
    /// Dumps the supported tools JSON
    DumpTools(UserArgsDump),
    Call,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    #[command(subcommand)]
    command: Commands,
}

struct DumpHandler {}

async fn send_tool_list_req(client: &OMcpClient) -> Result<()> {
    let msg = JsonRPCMessageBuilder::new().with_id(2).with_method("tools/list").build();
    client.send(&msg).await
}

#[async_trait]
impl EventHandlerTrait for DumpHandler {
    async fn event_handler(&self, msg: &JsonRPCMessage) -> Result<()> {
        let msg_string = serde_json::to_string_pretty(msg)?;
        println!("{msg_string}");
        // we're done
        Err(Error::Eof)
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

async fn main_dump_tool(args: &UserArgsDump) -> Result<()> {
    init_logger(args.verbose, args.debug)?;

    if args.verbose {
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
    }

    let builder = OMcpClientBuilder::new(&args.server, OMcpServerType::Sse);

    let builder = match &args.bearer {
        Some(v) => builder.with_bearer(v)?,
        None => builder,
    };

    let mut client = builder.build();

    client.connect().await?;

    info!("connected to {}", args.server);

    let dh = DumpHandler {};
    send_tool_list_req(&client).await?;

    let ret = client.event_loop(dh).await;

    let ret = match ret {
        Ok(_) => Ok(()),
        Err(Error::Eof) => Ok(()),
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    };

    // don't really care about this failing but we want
    // return whatever the event loop returned
    let _ = client.disconnect().await;

    ret
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    match args.command {
        Commands::DumpTools(d) => main_dump_tool(&d).await,
        Commands::Call => Err(Error::NotImplemented),
    }
}
