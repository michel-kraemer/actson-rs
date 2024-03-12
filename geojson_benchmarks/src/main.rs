use ::serde::Serialize;
use anyhow::{Context, Ok, Result};
use std::fs::File;
use std::future::Future;
use std::path::PathBuf;
use std::time::Instant;
use tokio::fs::{self};

mod actson;
mod serde;

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

        let sjr = bench_parser(&path, "Serde JSON (Value)", serde::bench_value).await?;
        benchmark_results.push(sjr);

        let sjs = bench_parser(&path, "Serde JSON (struct)", serde::bench_struct).await?;
        benchmark_results.push(sjs);

        let sjcd = bench_parser(
            &path,
            "Serde JSON (custom deserializer)",
            serde::bench_custom_deser,
        )
        .await?;
        benchmark_results.push(sjcd);

        let abr = bench_parser(&path, "Actson (BufReader)", actson::bench_bufreader).await?;
        benchmark_results.push(abr);

        let at = bench_parser(&path, "Actson (Tokio)", actson::bench_tokio).await?;
        benchmark_results.push(at);

        let att = bench_parser(&path, "Actson (Tokio, two tasks)", actson::tokio_twotasks).await?;
        benchmark_results.push(att);

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
