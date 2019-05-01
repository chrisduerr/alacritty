#!/bin/sh

repo="https://github.com/jwilm/alacritty"

repodir="$HOME/alacritty"
benchdir="$HOME/benchmarks"
resultdir="$HOME/benchmarks/results"

build="$HOME/.cargo/bin/cargo build --release"
run="./target/release/alacritty -e"

benchmarks="alt-screen-random-write scrolling"
max_bench_secs=15

export DISPLAY=":0"

# Make sure all dependencies are installed
if [ ! $(command -v vtebench) ]; then
    echo "vtebench: command not found"
    exit 1
fi
if [ ! $(command -v hyperfine) ]; then
    echo "hyperfine: command not found"
    exit 1
fi

# Make sure the latest version is present
if [ ! -d "$repodir" ]; then
    git clone "$repo" "$repodir"
fi

git -C "$repodir" fetch origin
git -C "$repodir" reset --hard origin/master

cd "$repodir"
$build

# Make sure output directory exists
mkdir -p "$benchdir"

# Generate benchmarks if they don't exist yet
for bench in $benchmarks; do
    out="$benchdir/$bench.vte"

    if [ -f "$out" ]; then
        continue
    fi

    $run /bin/sh -c \
        "vtebench --term xterm -h \$(tput lines) -w \$(tput cols) -b 999999999999999 $bench \
            | tee $out" &
    sleep $max_bench_secs
    pkill vtebench
done

# Make sure results directory exists
mkdir -p "$resultdir"

# Run benchmarks
for bench in $benchmarks; do
    out="$resultdir/$bench"
    mkdir -p "$out"
    hyperfine -w 1 -s basic --export-json "$out/$(date --iso-8601=sec --utc).json" \
        "$run cat $benchdir/$bench.vte"
done
