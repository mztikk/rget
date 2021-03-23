#!/bin/bash
for target in "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu"; do
    cargo build --release --target=$target
done
