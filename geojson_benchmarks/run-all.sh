#!/usr/bin/env bash

set -e

# list of benchmarks to run
benchmarks=(
  "actson-bufreader"
  "actson-tokio"
  "actson-tokio-twotasks"
  "serde-value"
  "serde-struct"
  "serde-custom-deser"
)

# build benchmarks
cargo b -r

# expand non-matching globs to null instead of themselves
shopt -s nullglob

# read list of input files into array
files=( "data/*json" )

# build hyperfine command
hfcmd="hyperfine -w 3 -r 5 --export-json results.json"
for file in $files; do
  size=$(wc -c $file | awk '{print $1}')
  for benchmark in ${benchmarks[@]}; do
    hfcmd+=" -n \"$benchmark|$file|$size\" \"../target/release/geojson_benchmarks $benchmark -i $file\""
  done
done

# run command
sh -c "$hfcmd"
