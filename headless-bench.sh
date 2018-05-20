#!/bin/bash

# Build the project to make `./target/release/alacritty` available
cargo build --release

xvfb="xvfb-run -s '-screen 0 1920x1080x24'"

# List with benchmarks that should be run
# Format:
#     "'name' 'num bytes'"
benchmarks=(\
    "'scrolling' '50000000'" \
    "'alt-screen-random-write' '250000000'" \
    "'scrolling-in-region --lines-from-bottom 1' '50000000'" \
    "'scrolling-in-region --lines-from-bottom 25' '50000000'" \
    "'unicode-random-write' '4000000'")

# Run all benchmarks with docker
for i in ${!benchmarks[@]}
do
    bench="${benchmarks[$i]}"
    echo "Running benchmark $bench"
    sudo docker run -v "$(pwd):/source" undeadleech/vtebench "cd /source && $xvfb ./target/release/alacritty -e bash ./bench.sh $bench"
done

# XXX TODO FIXME Remove
ls -lah

# Print results
echo "\nResults:"
find . -iname "*.out" | while read file
do
    echo "$file"
    cat "$file" | head -n 2 | tail -n 1 | sed 's/real.*0m//g'
    echo ""
done

