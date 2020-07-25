use anyhow::{Context, Result};
use neovim_lib::Value;
use std::convert::TryFrom;

pub enum Message {
    Sync,
    Unknown(String),
}

impl From<String> for Message {
    fn from(event: String) -> Self {
        match &event[..] {
            "sync" => Message::Sync,
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
pub struct Position {
    pub x: u64,
    pub y: u64,
}

impl TryFrom<&Value> for Position {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self> {
        let values = value.as_array().with_context(|| "invalid position value")?;

        // TODO here
        eprintln!("{:?}", values);

        Ok(Position {
            x: values[0]
                .as_u64()
                .with_context(|| "invalid position x value")?,
            y: values[1]
                .as_u64()
                .with_context(|| "invalid position y value")?,
        })
    }
}

#[derive(Debug)]
pub struct SyncPayload {
    pub bufnr: u64,
    pub height: u64,
    pub scroll: u64,
    pub pos: Position,
    pub select_start: Position,
    pub select_end: Position,
    pub lines: Vec<String>,
    pub locations: Vec<Location>,
    pub hunks: Vec<Hunk>,
}

impl TryFrom<Vec<Value>> for SyncPayload {
    type Error = anyhow::Error;

    fn try_from(values: Vec<Value>) -> Result<SyncPayload> {
        Ok(SyncPayload {
            bufnr: values[0].as_u64().with_context(|| "invalid bufnr field")?,
            height: values[1].as_u64().with_context(|| "invalid height field")?,
            scroll: values[2].as_u64().with_context(|| "invalid scroll field")?,
            pos: Position::try_from(&values[3])?,
            select_start: Position::try_from(&values[4])?,
            select_end: Position::try_from(&values[5])?,

            lines: values[6]
                .as_array()
                .with_context(|| "invalid lines field")?
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .with_context(|| format!("invalid line {}", value))
                })
                .collect::<Result<Vec<&str>>>()
                .with_context(|| "invalid lines value")?
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),

            locations: values[7]
                .as_array()
                .with_context(|| "invalid locations field")?
                .iter()
                .map(Location::try_from)
                .collect::<Result<Vec<Location>>>()
                .with_context(|| "invalid location value")?,

            hunks: values[8]
                .as_array()
                .with_context(|| "invalid hunks field")?
                .iter()
                .map(Hunk::try_from)
                .collect::<Result<Vec<_>>>()
                .with_context(|| "invalid hunk value")?,
        })
    }
}
