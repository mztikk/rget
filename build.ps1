foreach ($target in "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu") {
    cargo build --release --target=$target
}
