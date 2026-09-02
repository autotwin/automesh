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

## Optional

* [VS Code](https://code.visualstudio.com/) with the following extensions:
    * [Python](https://marketplace.visualstudio.com/items?itemName=ms-python.python)
    * [Python Debugger](https://marketplace.visualstudio.com/items?itemName=ms-python.debugpy)
    * [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
* [GitHub CLI](https://cli.github.com)
* **`cargo bi` build+install helper** (macOS) — [`scripts/cargo-bi`](https://github.com/autotwin/automesh/blob/main/scripts/cargo-bi)
  chains `cargo build` and `cargo install --path . --force` into one
  command; see [Development Cycle Overview](#development-cycle-overview)
  for why that matters.  Install it once, symlinked so future updates to
  the script are picked up automatically:

    ```sh
    ln -s "$(pwd)/scripts/cargo-bi" ~/.cargo/bin/cargo-bi
    ```

    Cargo's plugin mechanism treats any executable named `cargo-*` on
    `PATH` as the subcommand `cargo *`, so this makes `cargo bi` (and
    `cargo bi --release`) available immediately, from any directory
    containing a `Cargo.toml`.

## Clone Repository

```sh
git clone git@github.com:autotwin/automesh.git
cd automesh
```

## Building the Book Locally

The book embeds live command output via `mdbook-cmdrun`: most pages run
`automesh` (some piped through `ansifilter` to strip ANSI color codes for
plain-text embedding), and a few run `cat` or `python`.  If `automesh` isn't
resolvable on `PATH`, `mdbook-cmdrun` fails silently — the affected output
blocks simply render empty, with no error — so before running `mdbook build`
or `mdbook serve`, make sure both are available:

```sh
cd automesh               # the repository root, containing Cargo.toml
cargo install --path .    # installs `automesh` to ~/.cargo/bin; re-run after
                          # source changes you want reflected in the book
brew install ansifilter   # macOS, one-time (apt-get install ansifilter on Linux)
```

`cargo install --path .` must be run from the repository root (or pass that
path explicitly, e.g. `cargo install --path ~/autotwin/automesh` from
anywhere) — `--path` points at the directory containing the crate's
`Cargo.toml`, not at the book or any other subdirectory.

Both install locations are typically already on `PATH` via Rustup/Homebrew,
so no `PATH` changes should be needed.

## Development Cycle Overview

* **Branch**
* **Develop**
    * `cargo build`
        * `cargo install --path . --force` — updates the `automesh` on
          `PATH` (`~/.cargo/bin/automesh`).  `cargo build` alone only
          writes to `target/`; plain `automesh` keeps resolving to
          whatever was last installed until this is rerun.
        * `cargo bi` (or `cargo bi --release`) runs both steps in one
          command — see [Optional](#optional) for one-time setup.
    * Develop:
        * tests
        * implementation
    * Document:
        * `mdbook build` (see [Building the Book Locally](#building-the-book-locally)
          for prerequisites)
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
