# Development

## Prerequisites

* [Git](https://git-scm.com/)
* [CMake](https://cmake.org/download/) [^cmake_2024]

## Optional

* [VS Code](https://code.visualstudio.com/) with the following extensions:
    * [Python](https://marketplace.visualstudio.com/items?itemName=ms-python.python)
    * [Python Debugger](https://marketplace.visualstudio.com/items?itemName=ms-python.debugpy)
    * [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
* [GitHub CLI](https://cli.github.com)

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
    * Test:
        * `cargo test`
        * `cargo run` // test without required input and output flags
        * `cargo run --release -- -i tests/input/f.npy -o foo.exo`
        * `cargo run -- --help`
    * Pre-commit:
        * `pre-commit run --all-files`
    * Clean:
        * `cargo clean`
    * `cargo doc --open`
* **Test**
    * `maturin develop --release --features python`
* **Merge Request**

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
is known to the module system.  The administrator must add the root of your module directory
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

## References

[^cmake_2024]: As of Oct 2024, `cmake` is required for `hdf5-metno-src v0.9.2`, used for writing Exodus II files.  On macOS with `brew`, install with `brew install cmake` instead of the GUI installer.
