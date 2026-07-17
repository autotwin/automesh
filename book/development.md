# Development

[![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)
[![docs](https://img.shields.io/badge/Docs-API-e57300?logo=docsdotrs&logoColor=000000)](https://docs.rs/automesh)

## Prerequisites

* [Git](https://git-scm.com/)
* [Rust](https://www.rust-lang.org/) and Cargo, installed via [Rustup](https://rustup.rs):

    ```sh
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

    Rust updates occur every six weeks.  To update Rust:

    ```sh
    rustup update
    ```
* [CMake](https://cmake.org/download/) [^cmake_2024]

## Optional

* [VS Code](https://code.visualstudio.com/) with the following extensions:
    * [Python](https://marketplace.visualstudio.com/items?itemName=ms-python.python)
    * [Python Debugger](https://marketplace.visualstudio.com/items?itemName=ms-python.debugpy)
    * [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
* [GitHub CLI](https://cli.github.com)

## Clone Repository

```sh
git clone git@github.com:autotwin/automesh.git
cd automesh
```

## Development Cycle Overview

* **Branch**
* **Develop**
    * `cargo build`
    * Develop:
        * tests
        * implementation
    * Document:
        * `mdbook build`
            * output: `automesh/book/build`
        * `mdbook serve --open`
            * interactive mode
            * On the local machine, with **Firefox**, open the **`index.html`** file, e.g.,
                * `file:///Users/chovey/autotwin/automesh/book/build/index.html`
        * `cargo rustdoc --open -- --html-in-header docs/katex.html`
    * Test:
        * `cargo test`
        * `cargo run` // test without required input and output flags
        * `cargo run --release -- -i tests/input/f.npy -o foo.exo`
        * `cargo run -- --help`
    * Lint:
        * `cargo clippy`
    * Pre-commit:
        * `pre-commit run --all-files`
    * Clean:
        * `cargo clean`
* **Merge Request**

## References

[^cmake_2024]: As of Oct 2024, `cmake` is required for `hdf5-metno-src v0.9.2`, used for writing Exodus II files.  On macOS with `brew`, install with `brew install cmake` instead of the GUI installer.
