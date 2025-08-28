use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSON_RPC_VERSION: &str = "2.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRPCTool {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRPCServerInfo {
    pub name: String,
    pub version: String,
}

pub struct JsonRPCMessageBuilder {
    inner: JsonRPCMessage,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct JsonRPCMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "params")]
    pub parameters: Option<HashMap<String, Value>>,
}

impl JsonRPCMessage {}

impl Default for JsonRPCMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRPCMessageBuilder {
    pub fn new() -> Self {
        let inner = JsonRPCMessage {
            jsonrpc: JSON_RPC_VERSION.into(),
            ..Default::default()
        };

        Self { inner }
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.inner.id = Some(id);
        self
    }

    pub fn with_method<S>(mut self, method: S) -> Self
    where
        S: AsRef<str>,
    {
        self.inner.method = Some(method.as_ref().into());
        self
    }

    pub fn with_parameter(mut self, parameters: HashMap<String, Value>) -> Self {
        self.inner.parameters = Some(parameters);
        self
    }

    pub fn build(self) -> JsonRPCMessage {
        self.inner
    }
}

///////////////////////////////////////////////////////////////////////////////
// TEST
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    // Import everything from the parent module

    use crate::json_rpc::JsonRPCMessage;

    #[tokio::test]
    async fn parser() {
        let msg = r#"{
          "jsonrpc": "2.0",
          "id": 1,
          "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
              "experimental": {},
              "prompts": {
                "listChanged": false
              },
              "tools": {
                "listChanged": false
              }
            },
            "serverInfo": {
              "name": "home-assistant",
              "version": "1.5.0"
            }
          }
        }
"#;

        let rpc: JsonRPCMessage = serde_json::from_str(msg).unwrap();
        assert_eq!(rpc.jsonrpc, "2.0");
    }
}
