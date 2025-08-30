use clap::{Parser, Subcommand};
use log::{
    LevelFilter, {error, info},
};
use omcp::{
    client::{builder::OMcpClientBuilder, types::OMcpServerType},
    error::{Error, Result},
};

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
    /// List supported tools to JSON
    ListTools(UserArgsDump),
    Call,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UserArgs {
    #[command(subcommand)]
    command: Commands,
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

async fn main_list_tool(args: &UserArgsDump) -> Result<()> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = UserArgs::parse();

    match args.command {
        Commands::ListTools(d) => main_list_tool(&d).await,
        Commands::Call => Err(Error::NotImplemented),
    }
}
