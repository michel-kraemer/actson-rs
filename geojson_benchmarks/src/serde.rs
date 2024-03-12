use anyhow::Result;
use serde_json::Value;
use std::{collections::HashMap, fmt, fs::File, io::BufReader, marker::PhantomData, path::PathBuf};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

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

#[allow(dead_code)]
#[derive(Deserialize)]
struct CustomDeserFeatureCollection {
    r#type: String,

    #[serde(deserialize_with = "deserialize_features")]
    #[serde(rename(deserialize = "features"))]
    num_features: u64,
}

fn deserialize_features<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct FeatureVisitor<T>(PhantomData<fn() -> T>);

    impl<'de> Visitor<'de> for FeatureVisitor<u64> {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a nonempty sequence of numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<u64, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut count = 0u64;
            while (seq.next_element::<Feature>()?).is_some() {
                count += 1;
            }
            std::result::Result::Ok(count)
        }
    }

    let visitor = FeatureVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
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

pub async fn bench_custom_deser(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let _: CustomDeserFeatureCollection = serde_json::from_reader(reader)?;

    Ok(len)
}
