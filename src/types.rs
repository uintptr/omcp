use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Deserialize, Serialize)]
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
