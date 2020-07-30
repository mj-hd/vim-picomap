use anyhow::{Context, Result};
use neovim_lib::Value;
use std::convert::TryFrom;

pub enum Message {
    Sync,
    Show,
    Close,
    Resize,
    Unknown(String),
}

impl From<String> for Message {
    fn from(event: String) -> Self {
        match &event[..] {
            "sync" => Message::Sync,
            "show" => Message::Show,
            "close" => Message::Close,
            "resize" => Message::Resize,
            _ => Message::Unknown(event),
        }
    }
}

#[derive(Debug)]
pub enum LocationType {
    Unknown,
    Warning,
    Error,
}

impl From<String> for LocationType {
    fn from(value: String) -> Self {
        match &value[..] {
            "W" => Self::Warning,
            "E" => Self::Error,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Location {
    pub lnum: u64,
    pub typ: LocationType,
    pub text: String,
}

impl TryFrom<&Value> for Location {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        let fields = value.as_map().with_context(|| "invalid location value")?;

        Ok(Location {
            lnum: fields
                .iter()
                .find(|field| field.0.as_str() == Some("lnum"))
                .with_context(|| "missing location lnum")?
                .1
                .as_u64()
                .with_context(|| "invalid location lnum")?,
            typ: LocationType::from(
                fields
                    .iter()
                    .find(|field| field.0.as_str() == Some("type"))
                    .with_context(|| "missing location type")?
                    .1
                    .as_str()
                    .with_context(|| "invalid location type")?
                    .to_string(),
            ),

            text: fields
                .iter()
                .find(|field| field.0.as_str().unwrap() == "text")
                .with_context(|| "missing location text field")?
                .1
                .as_str()
                .with_context(|| "invalid location text")?
                .to_string(),
        })
    }
}

#[derive(Debug)]
pub struct Hunk {
    pub lnum: u64,
    pub len: usize,
}

impl TryFrom<&Value> for Hunk {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        let values = value.as_array().with_context(|| "invalid hunk value")?;

        Ok(Self {
            lnum: values[2].as_u64().with_context(|| "invalid hunk lnum")?,
            len: values[3].as_u64().with_context(|| "invalid hunk len")? as usize,
        })
    }
}

#[derive(Debug)]
pub struct SyncPayload {
    pub locations: Vec<Location>,
    pub hunks: Vec<Hunk>,
}

impl TryFrom<Vec<Value>> for SyncPayload {
    type Error = anyhow::Error;

    fn try_from(values: Vec<Value>) -> Result<SyncPayload> {
        Ok(SyncPayload {
            locations: values[0]
                .as_array()
                .with_context(|| "invalid locations field")?
                .iter()
                .map(Location::try_from)
                .collect::<Result<Vec<Location>>>()
                .with_context(|| "invalid location value")?,

            hunks: values[1]
                .as_array()
                .with_context(|| "invalid hunks field")?
                .iter()
                .map(Hunk::try_from)
                .collect::<Result<Vec<_>>>()
                .with_context(|| "invalid hunk value")?,
        })
    }
}
