use std::collections::HashMap;

use log::{error, info};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Stdin},
    select,
};

use crate::{
    error::{Error, Result},
    json_rpc::JsonRPCMessage,
    types::BakedMcpToolTrait,
};

const IO_BUFFER_SIZE: usize = 8 * 1024;

pub struct OmcpServer<E> {
    tools: HashMap<String, Box<dyn BakedMcpToolTrait<Error = E>>>,
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////
async fn read_all(stream: &mut BufReader<Stdin>) -> Result<String> {
    let mut vec: Vec<u8> = Vec::new();

    let mut buffer = vec![0u8; IO_BUFFER_SIZE];

    loop {
        match stream.read(&mut buffer).await {
            Ok(len) => {
                info!("len: {len}");

                if 0 == len {
                    return Err(Error::Eof);
                }

                vec.extend(&buffer[0..len]);

                if vec.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) => {
                error!("{e}");
                return Err(e.into());
            }
        }
    }

    Ok(String::from_utf8(vec)?)
}

async fn process_message(_msg: &JsonRPCMessage) -> Result<JsonRPCMessage> {
    unimplemented!()
}

////////////////////////////////////////////////////////////////////////////////
// IMPL
////////////////////////////////////////////////////////////////////////////////
impl<E> OmcpServer<E> {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn add_tool<S, T>(&mut self, name: S, client: T)
    where
        S: AsRef<str>,
        T: BakedMcpToolTrait<Error = E> + 'static,
    {
        let boxed_client = Box::new(client);
        self.tools.insert(name.as_ref().to_string(), boxed_client);
    }

    pub fn start(&mut self) -> Result<()> {
        Ok(())
    }

    pub async fn io_loop(&mut self) -> Result<()> {
        let mut stdin_reader = BufReader::new(io::stdin());
        let mut stdout_writer = BufWriter::new(io::stdout());

        loop {
            select! {
               local = read_all(&mut stdin_reader) => {
                   match local{
                       Ok(msg) => {

                           let req: JsonRPCMessage = match serde_json::from_str(&msg){
                               Ok(v) => v,
                               Err(e) => {
                                   error!("{e}");
                                   continue
                               }
                           };

                           let res = match process_message(&req).await{
                               Ok(v) => v,
                               Err(e) => {
                                   error!("{e}");
                                   continue
                               }
                           };

                           let res = match serde_json::to_string(&res){
                               Ok(v) => v,
                               Err(e) => {
                                   error!("{e}");
                                   continue
                               }
                           };

                           stdout_writer.write_all(res.as_bytes()).await?;
                       }
                       Err(e) => {
                           error!("{e}");
                           break Err(e.into())
                       }
                   }
               }
            }
        }
    }
}
