# Installation

Use `automesh` from one of the following interfaces:

* command line interface,
* Rust interface, or
* Python interface.

All interfaces are independent from each other:

* The Rust interfaces can be used without the Python interface.
* The Python interface can be used without the Rust interfaces.

For macOS and Linux, use a terminal.  For Windows, use a Command Prompt (CMD) or PowerShell.

## Step 1: Install Prerequisites

* The command line interface and Rust interface depend on [Rust](https://www.rust-lang.org/) and [Cargo](https://doc.rust-lang.org/cargo/).
  * Cargo is the Rust package manager.
  * Cargo is included with the Rust installation.
* The Python interface depends on [Python](https://www.python.org/) and [pip](https://pypi.org/project/pip/).
  * pip is the Python package installer.
  * pip is included with the standard installation of Python starting from Python 3.4.

### Rust Prerequisites

Install Rust and Cargo for your operating system:

#### macOS and Linux

1. Open a terminal.  Install Rust using [`rustup`](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Follow the on-screen instructions to complete the installation.
3. Add Cargo's bin directory to your `PATH`:

```sh
source $HOME/.cargo/env
```

#### Windows

1. Download and run the [Rust installer](https://rustup.rs).
2. Follow the installation instructions in the command prompt.  Additional Windows installation details are available in [The rustup book](https://rust-lang.github.io/rustup/installation/windows.html).
3. Ensure that Cargo's bin directory is in your `PATH`. You may need to restart your command prompt or system.

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
python3 -m venv .venv
```

Activate the virtual environment:

```sh
source .venv/bin/activate       # for bash shell
source .venv/bin/activate.csh   # for c shell
source .venv/bin/activate.fish  # for fish shell
.\.venv\Scripts\activate        # for powershell
```

## Step 2: Install `automesh`

Install the desired interface.

### Command Line Interface

[![book](https://img.shields.io/badge/automesh-Book-blue?logo=mdbook&logoColor=000000)](https://autotwin.github.io/automesh/cli)
[![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)

```sh
cargo install automesh
```

### Rust Interface

[![crates](https://img.shields.io/crates/v/automesh?logo=rust&logoColor=000000&label=Crates&color=32592f)](https://crates.io/crates/automesh)
[![docs](https://img.shields.io/badge/Docs-API-e57300?logo=docsdotrs&logoColor=000000)](https://docs.rs/automesh)

```sh
cargo add automesh
```

### Python Interface

[![pypi](https://img.shields.io/pypi/v/automesh?logo=pypi&logoColor=FBE072&label=PyPI&color=4B8BBE)](https://pypi.org/project/automesh)
[![docs](https://img.shields.io/badge/Docs-API-8CA1AF?logo=readthedocs)](https://automesh.readthedocs.io)

```sh
pip install automesh
```

## Step 3: Verify Installation

### Rust Interfaces

```sh
automesh
```

### Python Interface

```sh
python

# In Python, import the module
>>> import automesh

# List all attributes and methods of the module
>>> dir(automesh)

# Get help on the module
>>> help(automesh)
```
