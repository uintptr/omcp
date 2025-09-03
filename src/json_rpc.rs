use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JSON_RPC_VERSION: &str = "2.0";
pub const JSON_RPC_PROTOCOL_VERSION: &str = "2025-03-26";
pub const CLIENT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
pub struct JsonRPCRoots {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

#[derive(Serialize)]
pub struct JsonRPCSampling {}

#[derive(Serialize)]
pub struct JsonRPCCapabilities {
    roots: JsonRPCRoots,
    sampling: JsonRPCSampling,
}

#[derive(Serialize)]
pub struct JsonRPCClientInfo {
    name: String,
    version: String,
}

#[derive(Serialize)]
pub struct JsonRPCInitParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: JsonRPCCapabilities,
    #[serde(rename = "clientInfo")]
    client_info: JsonRPCClientInfo,
}

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
pub struct JsonRPCError {
    code: u64,
    message: String,
}

pub type JsonRPCParameters = HashMap<String, Value>;

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
    pub error: Option<JsonRPCError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "params")]
    pub parameters: Option<JsonRPCParameters>,
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl JsonRPCInitParams {
    pub fn new() -> Self {
        let roots = JsonRPCRoots { list_changed: true };

        let sampling = JsonRPCSampling {};

        let capabilities = JsonRPCCapabilities { roots, sampling };

        let client_info = JsonRPCClientInfo {
            name: CLIENT_NAME.to_string(),
            version: CLIENT_VERSION.to_string(),
        };

        Self {
            protocol_version: JSON_RPC_PROTOCOL_VERSION.to_string(),
            capabilities,
            client_info,
        }
    }
}

impl Default for JsonRPCInitParams {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRPCMessage {}

impl AsRef<JsonRPCMessage> for JsonRPCMessage {
    fn as_ref(&self) -> &JsonRPCMessage {
        self
    }
}

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

    pub fn with_result(mut self, result: HashMap<String, Value>) -> Self {
        self.inner.result = Some(result);
        self
    }

    pub fn with_error<S>(mut self, code: u64, message: S) -> Self
    where
        S: AsRef<str>,
    {
        let error = JsonRPCError {
            code,
            message: message.as_ref().to_string(),
        };

        self.inner.error = Some(error);
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
