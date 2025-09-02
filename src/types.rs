use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{Error, Result},
    json_rpc::JsonRPCParameters,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ToolType {
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "function")]
    Function,
}

pub type McpArguments = HashMap<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct McpParams {
    #[serde(rename = "name")]
    pub tool_name: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub arguments: McpArguments,
}

impl TryFrom<&McpParams> for JsonRPCParameters {
    type Error = Error;

    fn try_from(mcp_params: &McpParams) -> Result<JsonRPCParameters> {
        let mcp_params_json = serde_json::to_string(mcp_params)?;

        let params: JsonRPCParameters = serde_json::from_str(&mcp_params_json)?;

        Ok(params)
    }
}

impl McpParams {
    pub fn new<S>(name: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            tool_name: name.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn add_argument<S>(&mut self, name: S, value: Value)
    where
        S: AsRef<str>,
    {
        self.arguments.insert(name.as_ref().to_string(), value);
    }
}

impl AsRef<McpParams> for McpParams {
    fn as_ref(&self) -> &McpParams {
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpToolProperty {
    #[serde(rename = "type")]
    pub property_type: ToolType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<McpToolSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enums: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpToolSchema {
    #[serde(rename = "type")]
    pub schema_type: ToolType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, McpToolProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enums: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Option<McpToolSchema>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum McpTypes {
    #[serde(rename = "sse")]
    Sse,
}

impl std::fmt::Display for McpTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpTypes::Sse => write!(f, "sse"),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// IMPL
///////////////////////////////////////////////////////////////////////////////

impl FromStr for ToolType {
    type Err = Error;

    fn from_str(s: &str) -> Result<ToolType> {
        match s {
            "object" => Ok(ToolType::Object),
            "string" => Ok(ToolType::String),
            "integer" => Ok(ToolType::Integer),
            "boolean" => Ok(ToolType::Boolean),
            "array" => Ok(ToolType::Array),
            "number" => Ok(ToolType::Number),
            "function" => Ok(ToolType::Function),
            _ => Err(Error::NotImplemented),
        }
    }
}
