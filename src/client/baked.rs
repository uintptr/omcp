use crate::{
    client::{io::OMcpClientTrait, types::BakedMcpToolTrait},
    error::{Error, Result},
    types::{McpParams, McpTool},
};
use async_trait::async_trait;

pub struct BackedClient<E> {
    handler: Box<dyn BakedMcpToolTrait<Error = E>>,
}

impl<E: std::fmt::Display + 'static> BackedClient<E> {
    pub fn new<H>(handler: H) -> Box<dyn OMcpClientTrait>
    where
        H: BakedMcpToolTrait<Error = E> + 'static,
    {
        let b = Self {
            handler: Box::new(handler),
        };

        Box::new(b)
    }
}

#[async_trait(?Send)]
impl<E: std::fmt::Display> OMcpClientTrait for BackedClient<E> {
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }
    async fn disconnect(&mut self) -> Result<()> {
        Ok(())
    }
    async fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        Err(Error::NotImplemented)
    }
    async fn call(&mut self, mcp_params: &McpParams) -> Result<String> {
        match self.handler.call(mcp_params) {
            Ok(v) => Ok(v),
            Err(e) => {
                let err_msg = format!("{e}");
                Err(Error::FunctionCallFailure { error: err_msg })
            }
        }
    }
}
