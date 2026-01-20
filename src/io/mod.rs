use anyhow::{Context, Result};
use serde_json;
use std::fs;
use std::path::Path;

use crate::core::GraphDef;

pub struct GraphIO;

impl GraphIO {
    /// Load a graph definition from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<GraphDef> {
        let data = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {:?}", path.as_ref()))?;

        let graph_def: GraphDef =
            serde_json::from_str(&data).with_context(|| "Failed to parse JSON")?;

        Ok(graph_def)
    }

    /// Save a graph definition to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(graph_def: &GraphDef, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(graph_def)
            .with_context(|| "Failed to serialize graph definition")?;

        fs::write(&path, json)
            .with_context(|| format!("Failed to write file: {:?}", path.as_ref()))?;

        Ok(())
    }

    /// Load a graph definition from a JSON string
    pub fn from_json(json: &str) -> Result<GraphDef> {
        serde_json::from_str(json).with_context(|| "Failed to parse JSON string")
    }

    /// Convert a graph definition to JSON string
    pub fn to_json(graph_def: &GraphDef) -> Result<String> {
        serde_json::to_string_pretty(graph_def)
            .with_context(|| "Failed to serialize graph definition")
    }
}
