use actson::tokio::AsyncBufReaderJsonFeeder;
use anyhow::{Context, Ok, Result};
use geojson::FeatureCollection;
use serde::Serialize;
use serde_json::Value;
use std::future::Future;
use std::path::PathBuf;
use std::time::Instant;
use std::{fs::File, io::BufReader};
use tokio::fs::{self};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;

use actson::feeder::{BufReaderJsonFeeder, PushJsonFeeder};
use actson::{JsonEvent, JsonParser};

mod geojson;

#[derive(Serialize)]
struct BenchmarkFileResult<'a> {
    filename: String,
    len: u64,
    benchmark_results: Vec<BenchmarkResult<'a>>,
}

#[derive(Serialize)]
struct BenchmarkResult<'a> {
    name: &'a str,
    elapsed_seconds: f64,
    throughput_mb_per_sec: f64,
}

async fn bench_parser<'a, 'b, F, Fut>(
    path: &'a PathBuf,
    name: &'b str,
    run: F,
) -> Result<BenchmarkResult<'b>>
where
    F: FnOnce(&'a PathBuf) -> Fut,
    Fut: Future<Output = Result<u64>>,
{
    println!("{} ...", name);

    let start = Instant::now();
    let len = run(path).await?;
    let elapsed_seconds = start.elapsed().as_secs_f64();
    let throughput_mb_per_sec = len as f64 / 1000.0 / 1000.0 / elapsed_seconds;

    println!("{:.2?}s {:.2} MB/s", elapsed_seconds, throughput_mb_per_sec);

    let r = BenchmarkResult {
        name,
        elapsed_seconds,
        throughput_mb_per_sec,
    };

    Ok(r)
}

async fn bench_serde_json(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let _: Value = serde_json::from_reader(reader)?;

    Ok(len)
}

async fn bench_serde_json_struct(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let _: FeatureCollection = serde_json::from_reader(reader)?;

    Ok(len)
}

async fn bench_actson_bufreader(path: &PathBuf) -> Result<u64> {
    let file = File::open(path)?;
    let len = file.metadata()?.len();
    let reader = BufReader::new(file);

    let feeder = BufReaderJsonFeeder::new(reader);
    let mut parser = JsonParser::new(feeder);
    while let Some(event) = parser.next_event()? {
        match event {
            JsonEvent::NeedMoreInput => parser.feeder.fill_buf()?,

            // make sure all values are parsed
            JsonEvent::FieldName => _ = parser.current_str(),
            JsonEvent::ValueString => _ = parser.current_str(),
            JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
            JsonEvent::ValueFloat => _ = parser.current_float(),

            _ => {}
        }
    }

    Ok(len)
}

async fn bench_actson_tokio(path: &PathBuf) -> Result<u64> {
    let file = tokio::fs::File::open(path).await?;
    let len = file.metadata().await?.len();
    let reader = tokio::io::BufReader::new(file);

    let feeder = AsyncBufReaderJsonFeeder::new(reader);
    let mut parser = JsonParser::new(feeder);
    while let Some(event) = parser.next_event()? {
        match event {
            JsonEvent::NeedMoreInput => parser.feeder.fill_buf().await?,

            // make sure all values are parsed
            JsonEvent::FieldName => _ = parser.current_str(),
            JsonEvent::ValueString => _ = parser.current_str(),
            JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
            JsonEvent::ValueFloat => _ = parser.current_float(),

            _ => {} // do something useful with the event
        }
    }

    Ok(len)
}

async fn bench_actson_tokio_twotasks(path: &PathBuf) -> Result<u64> {
    let (tx, mut rx) = mpsc::channel(1);

    let mut file = tokio::fs::File::open(path).await?;
    let len = file.metadata().await?.len();

    let reader_task = tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 65 * 1024];
            let r = file.read(&mut buf).await?;
            if r == 0 {
                break;
            }
            buf.truncate(r);
            tx.send(buf).await?;
        }

        Ok(())
    });

    let parser_task = tokio::spawn(async move {
        let feeder = PushJsonFeeder::new();
        let mut parser = JsonParser::new(feeder);
        let mut i = 0;
        let mut buf = Vec::new();
        while let Some(event) = parser.next_event()? {
            match event {
                JsonEvent::NeedMoreInput => {
                    i += parser.feeder.push_bytes(&buf[i..]);
                    if i == buf.len() {
                        if let Some(b) = rx.recv().await {
                            buf = b;
                            i = 0;
                        } else {
                            parser.feeder.done();
                        }
                    }
                }

                // make sure all values are parsed
                JsonEvent::FieldName => _ = parser.current_str(),
                JsonEvent::ValueString => _ = parser.current_str(),
                JsonEvent::ValueInt => _ = parser.current_int::<i64>(),
                JsonEvent::ValueFloat => _ = parser.current_float(),

                _ => {} // do something useful with the event
            }
        }

        Ok(())
    });

    reader_task.await??;
    parser_task.await??;

    Ok(len)
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut files = fs::read_dir("data").await?;

    let mut file_results = Vec::new();
    while let Some(f) = files.next_entry().await? {
        let path = f.path();
        let Some(ext) = path.extension() else {
            continue;
        };

        if !ext.to_str().context("invalid extension")?.ends_with("json") {
            continue;
        }

        let path_str = path.to_str().context("invalid path")?.to_owned();
        println!("=== {} ...", path_str);

        let len = f.metadata().await?.len();
        let mut benchmark_results = Vec::new();

        benchmark_results.push(bench_parser(&path, "serde-json", bench_serde_json).await?);
        benchmark_results
            .push(bench_parser(&path, "serde-json-struct", bench_serde_json_struct).await?);
        benchmark_results
            .push(bench_parser(&path, "Actson (BufReader)", bench_actson_bufreader).await?);
        benchmark_results.push(bench_parser(&path, "Actson (Tokio)", bench_actson_tokio).await?);
        benchmark_results.push(
            bench_parser(
                &path,
                "Actson (Tokio, two tasks)",
                bench_actson_tokio_twotasks,
            )
            .await?,
        );

        file_results.push(BenchmarkFileResult {
            filename: path_str,
            len,
            benchmark_results,
        })
    }

    let output_file = File::create("results.json")?;
    serde_json::to_writer_pretty(output_file, &file_results)?;

    Ok(())
}
