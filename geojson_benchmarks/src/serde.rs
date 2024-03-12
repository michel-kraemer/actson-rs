use anyhow::Result;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FeatureCollection {
    r#type: String,
    features: Vec<Feature>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Feature {
    r#type: String,
    geometry: Geometry,
    properties: HashMap<String, PropertyValue>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Geometry {
    r#type: String,
    coordinates: Vec<Vec<Vec<f64>>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Int(i64),
}

pub async fn bench_value(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let _: Value = serde_json::from_reader(reader)?;

    Ok(len)
}

pub async fn bench_struct(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let _: FeatureCollection = serde_json::from_reader(reader)?;

    Ok(len)
}
