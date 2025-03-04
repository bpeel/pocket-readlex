#!/bin/bash

set -eu

rm -f "$1"

src_dir=$(dirname "$0")

case "$3" in
    "x86_64")
        target="x86_64-linux-android"
        ;;
    "x86")
        target="i686-linux-android"
        ;;
    "arm64")
        target="aarch64-linux-android"
        ;;
    "arm")
        target="armv7-linux-androideabi"
        ;;
    *)
        echo "Unknown target: $3" >&2
        exit 1
        ;;
esac

args=( \
       build \
           --manifest-path="$src_dir/../compiledb/Cargo.toml" \
           --lib \
           --target "$target" \
    )

if test "$2" != "Debug"; then
    args=("${args[@]}" --release)
    mode="release"
else
    mode="debug"
fi

linker_target="$(echo "$target" | sed s/armv7-/armv7a-/g)"

linker="$(dirname "$4")/${linker_target}21-clang"

if ! test -f "$linker"; then
    echo "Missing linker $linker" >&1
    exit 1
fi

export RUSTFLAGS=-Clinker="$linker"

cargo "${args[@]}"

cp "$src_dir/../compiledb/target/$target/$mode/libcompiledb.so" "$1"
