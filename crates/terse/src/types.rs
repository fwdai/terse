use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Rules,
    Nlp,
    Llm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenMethod {
    Chars,
    Tiktoken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressConfig {
    pub tiers: Vec<Tier>,
    pub token_method: TokenMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResult {
    pub text: String,
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub saved_tokens: usize,
    pub saved_percent: i64,
}

/// The content of a message, which may be a plain string, null, or a blocks array.
#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    Null,
    Blocks(Vec<Value>),
}

impl MessageContent {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

impl Serialize for MessageContent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            MessageContent::Text(s) => serializer.serialize_str(s),
            MessageContent::Null => serializer.serialize_none(),
            MessageContent::Blocks(b) => b.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for MessageContent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Value::deserialize(deserializer)?;
        match v {
            Value::Null => Ok(MessageContent::Null),
            Value::String(s) => Ok(MessageContent::Text(s)),
            Value::Array(a) => Ok(MessageContent::Blocks(a)),
            other => Ok(MessageContent::Text(other.to_string())),
        }
    }
}

/// A raw message as it comes from JSON.
/// Using `Value` for content avoids serde flatten + untagged enum conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: Option<MessageContent>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStats {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub saved_tokens: usize,
    pub saved_percent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithStats {
    pub role: String,
    pub content: Option<MessageContent>,
    #[serde(rename = "_stats")]
    pub stats: MessageStats,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStats {
    pub total_original_tokens: usize,
    pub total_compressed_tokens: usize,
    pub total_saved_tokens: usize,
    pub total_saved_percent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub messages: Vec<MessageWithStats>,
    pub stats: HistoryStats,
}
