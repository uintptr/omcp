use async_trait::async_trait;
use omcp::{
    error::{Error, Result},
    server::matrix::OmcpServer,
    types::{BakedMcpToolTrait, McpParams},
};
use rstaples::logging::StaplesLogger;

struct UnameTool {}

#[async_trait(?Send)]
impl BakedMcpToolTrait for UnameTool {
    type Error = Error;

    async fn call(&mut self, _params: &McpParams) -> Result<String> {
        unimplemented!()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    StaplesLogger::new()
        .with_log_level(log::LevelFilter::Info)
        .with_stderr()
        .start()?;

    let mut server = OmcpServer::<Error>::new();

    let uname_tool = UnameTool {};

    server.add_tool("uname", uname_tool);
    server.io_loop().await
}
