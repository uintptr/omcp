use crate::{
    client::{builder::OMcpClientBuilder, types::BakedMcpTool},
    error::{Error, Result},
    json_rpc::JsonRPCMessage,
    types::McpTool,
};

pub struct BackedClient {
    baked_tools: Vec<Box<dyn BakedMcpTool>>,
}

impl BackedClient {
    pub fn from_builder(builder: OMcpClientBuilder) -> Self {
        BackedClient {
            baked_tools: builder.baked_tools,
        }
    }

    pub fn send_message(&self, _msg: &JsonRPCMessage) -> Result<()> {
        todo!()
    }

    pub fn list_tools(&self) -> Result<Vec<McpTool>> {
        todo!()
    }

    pub fn call_tool<S>(&mut self, name: S) -> Result<String>
    where
        S: AsRef<str>,
    {
        for t in self.baked_tools.iter_mut() {
            if t.implements(name.as_ref()) {
                return t.call(name.as_ref());
            }
        }
        Err(Error::NotFound)
    }
}
