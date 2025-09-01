use clap::{Parser, Subcommand};
use log::{
    LevelFilter, {error, info},
};
use omcp::{
    client::{
        baked::BackedClient,
        builder::OMcpClientBuilder,
        io::OMcpClientTrait,
        types::{BakedMcpTool, OMcpServerType},
    },
    error::{Error, Result},
    types::McpParams,
};

use rstaples::{logging::StaplesLogger, staples::printkv};
use serde::Serialize;
use serde_json::Value;
use uname::uname;

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

#[derive(Parser)]
struct UserArgsCall {
    /// Server
    #[arg(short, long)]
    server: String,

    /// Bearer Token
    #[arg(short, long)]
    bearer: Option<String>,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Tool name
    #[arg(short, long)]
    tool: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List supported tools to JSON
    ListTools(UserArgsDump),
    BakedUname,
    Call(UserArgsCall),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Serialize)]
struct BakedUname {
    sys_name: String,
    node_name: String,
    release: String,
    version: String,
    machine: String,
}

impl BakedUname {
    pub fn new() -> Result<Self> {
        let info = uname()?;

        Ok(Self {
            sys_name: info.sysname,
            node_name: info.nodename,
            release: info.release,
            version: info.version,
            machine: info.machine,
        })
    }

    fn to_json(&self) -> Result<String> {
        let json_str = serde_json::to_string_pretty(&self)?;
        Ok(json_str)
    }
}

impl BakedMcpTool for BakedUname {
    type Error = Error;

    fn call(&mut self, _params: &McpParams) -> Result<String> {
        self.to_json()
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

async fn main_list_tool(args: UserArgsDump) -> Result<()> {
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

    let builder = OMcpClientBuilder::new(OMcpServerType::Sse).with_sse_url(&args.server);

    let builder = match &args.bearer {
        Some(v) => builder.with_sse_bearer(v)?,
        None => builder,
    };

    let mut client = builder.build();

    client.connect().await?;

    info!("connected to {}", args.server);

    let ret = client.list_tools().await;

    let ret = match ret {
        Ok(v) => {
            let json_str = serde_json::to_string_pretty(&v)?;

            println!("{json_str}");

            Ok(())
        }
        Err(e) => {
            error!("{e}");
            Err(e)
        }
    };

    // don't really care aboutthis failing but we want
    // return whatever the event loop returned
    if let Err(e) = client.disconnect().await {
        error!("{e}");
    }

    ret
}

async fn main_call(args: UserArgsCall) -> Result<()> {
    init_logger(args.verbose, false)?;

    if args.verbose {
        println!("MCP Dumper");
        printkv("Server", &args.server);
        printkv("Verbose", args.verbose);

        if let Some(b) = &args.bearer {
            // only display a little bit of the token to stdout
            if let Some(first_five) = b.get(0..5) {
                let first_five = format!("{}...", first_five);
                printkv("Bearer", first_five)
            }
        }
    }

    let builder = OMcpClientBuilder::new(OMcpServerType::Sse).with_sse_url(&args.server);

    let builder = match &args.bearer {
        Some(v) => builder.with_sse_bearer(v)?,
        None => builder,
    };

    let mut client = builder.build();

    client.connect().await?;

    info!("connected to {}", args.server);

    let mut params = McpParams::new("HassTurnOff");
    params.add_argument("area", Value::String("basement office".into()));

    let domain = vec![Value::String("light".into())];

    params.add_argument("domain", Value::Array(domain));

    let results = client.call(&params).await;

    let ret = match results {
        Ok(v) => {
            println!("{v}");
            Ok(())
        }
        Err(e) => Err(e),
    };

    if let Err(e) = client.disconnect().await {
        error!("{e}");
    }

    ret
}

async fn main_baked_uname() -> Result<()> {
    let uname = BakedUname::new()?;

    init_logger(true, true)?;

    let mut client = BackedClient::new(uname);

    let params = McpParams::new("uname");

    match client.call(&params).await {
        Ok(v) => {
            println!("{v}");
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    match args.command {
        Commands::ListTools(d) => main_list_tool(d).await,
        Commands::BakedUname => main_baked_uname().await,
        Commands::Call(c) => main_call(c).await,
    }
}
