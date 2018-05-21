#!/bin/bash

# Build the project to make `./target/release/alacritty` available
cargo build --release

xvfb="xvfb-run -a -s '-screen 0 1920x1080x24'"

# List with benchmarks that should be run
# Format:
#     "'name' 'num bytes'"
benchmarks=(\
    "'scrolling' '5000000' 'scrolling'" \
    "'alt-screen-random-write' '150000000' 'alt-screen-random-write'" \
    "'scrolling-in-region --lines-from-bottom 1' '5000000' 'scrolling-in-region-1'" \
    "'scrolling-in-region --lines-from-bottom 50' '5000000' 'scrolling-in-region-50'" \
    "'unicode-random-write' '10000000' 'unicode-random-write'")

# Run all benchmarks with docker
for i in ${!benchmarks[@]}
do
    bench="${benchmarks[$i]}"
    echo "Running benchmark $bench"
    docker_id=$(sudo docker run -d -v "$(pwd):/source" undeadleech/vtebench \
        "cd /source && $xvfb ./target/release/alacritty -e bash ./bench.sh $bench")
    sudo docker wait $docker_id
done


find . -maxdepth 1 -iname "*.json" | while read file
do
    echo "$file: "
    cat "$file"
done

