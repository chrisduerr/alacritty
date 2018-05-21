#!/bin/bash

# Example usage:
#     ./bench.sh scrolling 5000000
BENCH=$1
BYTES=$2
NAME=$3

# Generate requested benchmark
vtebench -w $(tput cols) -h $(tput lines) -sb $2 $1 > "/bench.vte"

# Run the benchmark and write output to `$BENCH.json`
hyperfine --print-stdout --export-json "/source/$NAME.json" "cat /bench.vte"

