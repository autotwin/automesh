# Python Development

[![pypi](https://img.shields.io/pypi/v/automesh?logo=pypi&logoColor=FBE072&label=PyPI&color=4B8BBE)](https://pypi.org/project/automesh)
[![docs](https://img.shields.io/badge/Docs-API-8CA1AF?logo=readthedocs)](https://automesh.readthedocs.io)

## Install Python

[Install](https://www.python.org/downloads/) a Python version [supported](https://github.com/autotwin/automesh/blob/main/pyproject.toml) by `automesh`.
[Install Rust](rust.md) as well.

## Create a Virtual Environment

Note: If a virtual environment folder `automesh/.venv` already exists from previous installs, then remove it as follows:

```sh
cd automesh                     # from the automesh directory
(.venv) deactivate              # if the virtual environment is currently active
rm -rf .venv                    # remove the virtual environment folder
                                # with `rm -rf .venv/`.

python -m venv .venv            # create the virtual environment

# activate the venv with one of the following:
source .venv/bin/activate       # for bash shell
source .venv/bin/activate.csh   # for c shell
source .venv/bin/activate.fish  # for fish shell
.\.venv\Scripts/activate        # for powershell

pip install --upgrade pip

pip install maturin
```

## Build and Test the Source Code

```sh
maturin develop --features python --extras dev

pytest
```

## Build a '.whl` file release

```sh
maturin build --release --features python
```

## Lint the Source Code

```sh
cargo clippy --features python

pycodestyle --exclude=.venv .  # exclude the .venv folder
```

## Build and Open the API Documentation

```sh
pdoc automesh --math --no-show-source --template-directory docs/
```
