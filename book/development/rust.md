# Rust Development

[![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)
[![docs](https://img.shields.io/badge/Docs-API-e57300?logo=docsdotrs&logoColor=000000)](https://docs.rs/automesh)

## Install Rust

Install Rust using `rustup` with the default standard installation:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Rust updates occur every six week. To update Rust:

```sh
rustup update
```

## Clone Repository

```sh
git clone git@github.com:autotwin/automesh.git
```

## Build and Test the Source Code

```sh
cd automesh
cargo test
```

## Lint the Source Code

```sh
cargo clippy
```

## Build and Open the API Documentation

```sh
cargo rustdoc --open -- --html-in-header docs/katex.html
```

## Run the Benchmarks

Execute `benches/block.sh` once, and subsequently run

```sh
rustup run nightly cargo bench
```
