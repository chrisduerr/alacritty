#!/bin/bash

# Example usage:
#     ./bench.sh scrolling 5000000
BENCH=$1
BYTES=$2

# Generate requested benchmark
vtebench -w $(tput cols) -h $(tput lines) -sb $2 $1 > "./$BENCH.vte"

# Run the benchmark and write output to `$BENCH-bench.out`
{ time cat "./$BENCH.vte"; } 2> "/source/$BENCH-bench.out"
