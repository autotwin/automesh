# Installation

`automesh` is a single command line program.  There is no Python API — there
is only one `automesh`, and every way of installing it produces the exact
same command line interface (CLI), with the same subcommands, same flags,
same output.

There are two independent, equivalent ways to get `automesh` onto your machine:

* **Rust**, via `cargo install automesh`, which compiles the binary from
  the source code, or
* **Python**, via `pipx install automesh` (or `pip install automesh`), which
  installs a prebuilt binary through PyPI.

Neither depends on the other — you don't need Rust installed to use the
Python route, and you don't need Python installed to use the Rust route.
Pick whichever toolchain you already have set up.  The Python route exists
for exactly one reason: it lets someone who already has Python and pip on
their machine — a data scientist or researcher working with segmentation
data, for example — get the `automesh` CLI without installing Rust and
Cargo first.  It is not a Python library; `import automesh` will not work.
See [Step 2](#step-2-install-automesh) for the details of what the Python
route actually installs.

For macOS and Linux, use a terminal.  For Windows, use a Command Prompt (CMD) or PowerShell.

Some macOS users have encountered a build error with the `netcdf-src` crate.  See [Troubleshooting](#troubleshooting) for a solution to this error.

## Step 1: Install Prerequisites

* The Rust route depends on [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/).
  * Cargo is the Rust package manager.
  * Cargo is included with the Rust installation.
* The Python route depends on [Python](https://www.python.org/) and [pip](https://pypi.org/project/pip/), and works best with [pipx](https://pipx.pypa.io/), which is the standard tool for installing Python-packaged command line applications.
  * pip is included with the standard installation of Python starting from Python 3.4.
  * pipx itself is installed via pip: `pip install pipx` (or `brew install pipx` on macOS, `sudo apt install pipx` on Debian/Ubuntu).

### Rust Prerequisites

It is recommended to install Rust using [Rustup](https://rust-lang.org/learn/get-started/), which is an installer and version management tool.

### Python Prerequisites

#### macOS

1. **Install Homebrew** (if you don't have it already).  Open the Terminal and run:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

2. **Install Python**.  After Homebrew is installed, run:

```sh
brew install python
```

3. **Verify** Python and pip are installed:

```sh
python3 --version
pip3 --version
```

#### Linux

1. **Update Package List**.  Open a terminal and run:

```sh
sudo apt update
```

2. **Install Python and pip**.  For Ubuntu or Debian-based systems, run:

```sh
sudo apt install python3 python3-pip
```

3. **Verify** Python and pip are installed:

```sh
python3 --version
pip3 --version
```

#### Windows

1. **Download Python**.  Go to the [official Python website](https://www.python.org/downloads/) and download the latest version of Python for Windows.
2. **Run the Installer**. During installation, make sure to check the box that says "Add Python to PATH."
3. **Verify** Python and pip are installed:

```sh
python --version
pip --version
```

#### All Environments

On all environments, a [virtual environment](https://docs.python.org/3/tutorial/venv.html) is recommended, but not required.  Create a virtual environment:

```sh
python3 -m venv .venv  # venv, or
uv venv .venv          # using uv
```

[`uv`](https://docs.astral.sh/uv/) is a fast Python package manager, written in Rust.  It is an alternative to `pip`.

Activate the virtual environment:

```sh
source .venv/bin/activate       # for bash shell
source .venv/bin/activate.csh   # for c shell
source .venv/bin/activate.fish  # for fish shell
.\.venv\Scripts\activate        # for powershell
```

## Step 2: Install `automesh`

Install with either route — both put the same `automesh` binary on your
`PATH`.

### Rust: build from source with Cargo

[![book](https://img.shields.io/badge/automesh-Book-blue?logo=mdbook&logoColor=000000)](https://autotwin.github.io/automesh/cli)
[![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)

```sh
cargo install automesh
```

Cargo downloads the source from [crates.io](https://crates.io/crates/automesh)
and compiles it locally.

### Python: install a prebuilt binary with pip

[![pypi](https://img.shields.io/pypi/v/automesh?logo=pypi&logoColor=FBE072&label=PyPI&color=4B8BBE)](https://pypi.org/project/automesh)

```sh
pipx install automesh    # recommended, or
pip install automesh     # using pip, or
uv pip install automesh  # using uv
```

`automesh`'s [PyPI project](https://pypi.org/project/automesh) publishes one
prebuilt wheel per supported platform, plus a source distribution as a
fallback for anything else.  As of version `0.4.1`, the published files are:

| file | contents |
| :--- | :--- |
| `automesh-0.4.1-py3-none-macosx_11_0_arm64.whl` | compiled binary for Apple Silicon macOS |
| `automesh-0.4.1-py3-none-manylinux_2_38_x86_64.whl` | compiled binary for x86_64 Linux |
| `automesh-0.4.1-py3-none-win_amd64.whl` | compiled binary for 64-bit Windows |
| `automesh-0.4.1.tar.gz` | source distribution, built locally with Cargo if no wheel matches your platform |

Each wheel's `py3-none-<platform>` tag is a tell that this isn't a normal
Python extension module — a real compiled Python module (built with, say,
`pyo3`) is tagged with a specific interpreter ABI, like
`cp312-cp312-macosx_...`.  `py3-none` means "works with any CPython 3, no
Python ABI dependency at all," which is exactly what you'd expect from a
wheel that contains nothing but a native executable.  That's what
[`maturin`](https://www.maturin.rs/)'s `bindings = "bin"` mode does: it
compiles the ordinary Rust binary, then packages it inside a wheel the same
way `pip` packages any console-script entry point, and installs it straight
into your environment's `bin/` (or `Scripts/` on Windows) directory — no
Python import machinery is ever involved.

`pipx` is recommended over plain `pip install` because `automesh` is an
application, not a library you'd import into other Python code; `pipx`
installs it into its own isolated environment and adds it to your `PATH`,
the same way you'd expect a CLI tool to be installed, without needing to
manage a virtual environment yourself.

## Step 3: Verify Installation

Whichever route you used, verification is identical — it's the same
program either way.

Run the command line help:

```sh
automesh
```

which should display the following:

```sh
<!-- cmdrun automesh --help -->
```

There is no Python module to `import`.  If you installed with `pipx`/`pip`
and want to call `automesh` from a Python script, invoke it as a
subprocess, exactly as you would a Cargo-installed copy:

```python
import subprocess

subprocess.run(["automesh", "mesh", "hex", "-i", "in.npy", "-o", "out.exo", "-r", "0"])
```

## Troubleshooting

### `ZLIB` target not found

Some users have encountered an error when trying to build the `netcdf` crate, e.g.,

```bash
...
error: failed to run custom build command for `netcdf-src v0.4.3`
...
The link interface of target "hdf5-static" contains:

  ZLIB::ZLIB

but the target was not found.  Possible reasons include:

  * There is a typo in the target name.
  * A find_package call is missing for an IMPORTED target.
  * An ALIAS target is missing.
...
```

HDF5 is looking for `ZLIB::ZLIB` as a CMake target, but it's not being found
even though `ZLIB` is present.

A solution is to use a dynamically-linked `netcdf` from Homebrew instead of
trying to build it statically.  This should avoid the CMake `ZLIB::ZLIB` target
issue entirely.

Update the `Cargo.toml` for `automesh` to avoid static linking:

```toml
...
# netcdf = { version = "=0.11.1", features = ["ndarray", "static"] }
netcdf = { version = "=0.11.1", features = ["ndarray"] }
...
```

Then,

```bash
# Make sure netcdf is installed via Homebrew
brew install netcdf

# Set the environment variable to use system netcdf
export NETCDF_DIR=/opt/homebrew

# Clean and build
cargo clean
cargo build
```

If you still encounter errors about `netcdf` not being found, also try:

```bash
export PKG_CONFIG_PATH=/opt/homebrew/lib/pkgconfig
export DYLD_LIBRARY_PATH=/opt/homebrew/lib
```

## Environment Modules

The **`automesh`** application can be installed as a service on a **High-Performance Computing (HPC)** system with the following steps:

### Install and Compile Application

**`automesh`** must be available and installed to a file location that is accessible by all compute nodes and users who need it.

* **Location:** Choose a central directory such as `/opt/hpc/` or `/sw/` for the **`automesh`** binaries, libraries, and associated files. For example, let's assume the install path is `/opt/hpc/apps/automesh/0.3.7`.
* **Compilation:** Compile **`automesh`** and all its dependencies statically if possible, or ensure all shared libraries (`.so` files) are also included in the installation directory structure.

### Create a Module File

The **module file**, a small script usually written in **Tcl** or **Lua**, provides the **`module load`** functionality. It tells the shell what changes to make to the user's environment when the module is loaded.
* **Location:** Module files are placed in a specific directory **structure** that is scanned by the **Environment Modules** software (e.g., Lmod or Tcl-based modules). A common path would be `/opt/hpc/modules/automesh/0.3.7`.

A typical module file would be something like this:

```sh
#%Module
# Define the application name and version
set name automesh
set version 0.3.7

# 1. Prerequisite check (e.g., automesh needs a specific compiler)
# If your app needs a specific compiler, you can ensure it's loaded first:
# prereq gcc/11.2

# 2. Update the PATH variable
# This is the most critical step, allowing the user to run 'automesh' command
prepend-path PATH /opt/hpc/apps/$name/$version/bin

# 3. Update the LD_LIBRARY_PATH variable
# Allows the application to find shared libraries if not statically linked
prepend-path LD_LIBRARY_PATH /opt/hpc/apps/$name/$version/lib

# 4. Define other environment variables (optional)
# For configuration files, data paths, etc.
setenv MYAPP_HOME /opt/hpc/apps/$name/$version

# 5. Provide a short description (optional)
module-whatis "Loads $name $version, a high-performance compute application."
```

### System Configuration

Finally, an HPC administrator needs to ensure that the directory containing the new module file
is known to the module system.  The administrator must add the root of your module directory
(e.g., `/opt/hpc/modules`) to the central module configuration, typically via a command such as:

```bash
module use /opt/hpc/modules
```

This is usually done in a global system profile script so it is active for all users.

### End User

Users can discover and load `automesh`:

* **Check for the module:** `module avail automesh`
* **Load the service:** `module load automesh/0.3.7` (or `module load automesh` if it is the default)
* **Run the program:** `automesh --version`
