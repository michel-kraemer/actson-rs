use std::collections::HashMap;

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct FeatureCollection {
    #[serde(rename(deserialize = "type"))]
    tpe: String,
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
