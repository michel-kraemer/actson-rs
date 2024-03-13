use ::serde::Serialize;
use anyhow::{Ok, Result};
use clap::{Args, Parser, Subcommand};
use std::future::Future;
use std::time::Instant;

mod actson;
mod serde;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read the input file with a `BufReader` and parse the read bytes with Actson
    ActsonBufreader(RunArgs),

    /// Use Tokio to asynchronously read the input file and parse it with Actson
    ActsonTokio(RunArgs),

    /// Use two Tokio tasks: one that reads the file asynchronously and one that parses the read bytes with Actson
    ActsonTokioTwotasks(RunArgs),

    /// Parse the JSON file with Serde JSON into a `Value`
    SerdeValue(RunArgs),

    /// Deserialize the JSON file with Serde JSON into a `struct`
    SerdeStruct(RunArgs),

    /// Use a custom Serde deserializer to prevent having to load the whole file into memory
    SerdeCustomDeser(RunArgs),
}

#[derive(Args)]
struct RunArgs {
    /// The path to the GeoJSON file to parse
    #[arg(short, long)]
    input: String,
}

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
    path: &'a str,
    name: &'b str,
    run: F,
) -> Result<BenchmarkResult<'b>>
where
    F: FnOnce(&'a str) -> Fut,
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
    let cli = Cli::parse();

    match &cli.command {
        Commands::ActsonBufreader(RunArgs { input }) => {
            bench_parser(input, "Actson (BufReader)", actson::bench_bufreader).await?;
        }
        Commands::ActsonTokio(RunArgs { input }) => {
            bench_parser(input, "Actson (Tokio)", actson::bench_tokio).await?;
        }
        Commands::ActsonTokioTwotasks(RunArgs { input }) => {
            bench_parser(input, "Actson (Tokio, two tasks)", actson::tokio_twotasks).await?;
        }
        Commands::SerdeValue(RunArgs { input }) => {
            bench_parser(input, "Serde JSON (Value)", serde::bench_value).await?;
        }
        Commands::SerdeStruct(RunArgs { input }) => {
            bench_parser(input, "Serde JSON (struct)", serde::bench_struct).await?;
        }
        Commands::SerdeCustomDeser(RunArgs { input }) => {
            bench_parser(
                input,
                "Serde JSON (custom deserializer)",
                serde::bench_custom_deser,
            )
            .await?;
        }
    }

    Ok(())
}
