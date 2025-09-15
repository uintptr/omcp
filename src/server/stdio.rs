use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
};

use crate::{
    error::{Error, Result},
    json_rpc::{JsonRPCMessage, JsonRPCMessageBuilder},
    server::types::OMcpServerTrait,
    types::McpParams,
};
use async_trait::async_trait;
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
};

#[derive(Default)]
pub struct StdioServer {
    pub program: PathBuf,
    pub cwd: PathBuf,
    pub args: Vec<String>,
    pub child: Option<Child>,
    pub rpc_msg_id: u64,
}

impl StdioServer {
    pub fn new<P>(program: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let cwd = env::current_dir()?;

        Ok(Self {
            program: program.as_ref().to_path_buf(),
            cwd,
            args: Vec::new(),
            ..Default::default()
        })
    }

    pub fn with_args(&mut self, args: &Vec<String>) {
        self.args = args.clone();
    }

    pub fn with_arg<S>(&mut self, arg: S)
    where
        S: AsRef<str>,
    {
        self.args.push(arg.as_ref().to_string());
    }

    pub fn set_working_directory<P>(&mut self, cwd: P)
    where
        P: AsRef<Path>,
    {
        self.cwd = cwd.as_ref().to_path_buf();
    }

    async fn send(&mut self, message: &[u8]) -> Result<()> {
        match self.child.as_mut() {
            Some(child) => match child.stdin.as_mut() {
                Some(stdin) => {
                    stdin.write_all(message).await?;
                    Ok(())
                }
                None => Err(Error::UrlNotInitialized),
            },
            None => Err(Error::UrlNotInitialized),
        }
    }

    async fn recv(&mut self) -> Result<String> {
        match self.child.as_mut() {
            Some(child) => match child.stdout.as_mut() {
                Some(_stdout) => Ok("".into()),
                None => Err(Error::UrlNotInitialized),
            },
            None => Err(Error::UrlNotInitialized),
        }
    }
}

#[async_trait(?Send)]
impl OMcpServerTrait for StdioServer {
    async fn listen(&mut self) -> Result<()> {
        let child = Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        self.child = Some(child);
        Ok(())
    }
    async fn close(&mut self) -> Result<()> {
        match self.child.take() {
            Some(mut child) => {
                child.kill().await?;
                child.wait().await?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    async fn list_tools(&mut self) -> Result<String> {
        unimplemented!()
    }

    async fn call(&mut self, params: &McpParams) -> Result<String> {
        let msg_id = self.rpc_msg_id;
        self.rpc_msg_id += 1;

        let req = JsonRPCMessageBuilder::new()
            .with_id(msg_id)
            .with_method(&params.tool_name)
            .with_parameter(params.arguments.clone())
            .build();

        let json_request = serde_json::to_string(&req)?;
        self.send(json_request.as_bytes()).await?;

        let json_response = self.recv().await?;

        let mut res: JsonRPCMessage = serde_json::from_str(&json_response)?;

        let data = res.result.take().ok_or(Error::Empty)?;

        let data_str = serde_json::to_string(&data)?;

        Ok(data_str)
    }
}
