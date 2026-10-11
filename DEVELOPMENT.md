# Development

Avulto is written in Rust and implemented using
[PyO3](https://github.com/PyO3/pyo3), and uses
[maturin](https://www.maturin.rs/) for development. To build and install
locally:

## Install Prerequisites
Rust can be installed at [rustup.sh](https://rustup.rs/). Python can be installed at [python.org](https://www.python.org/downloads/).

## Virtual Environment
Using a virtual environment will isolate project dependencies, preventing versioning conflicts, and random packages filling up your global python system.

Bash:
```sh
python -m venv .venv
source .venv/Scripts/activate
```

Powershell:
```sh
python -m venv .venv
.\.venv\Scripts\Activate.ps1
```

## Installation

```sh
# Install Development packages
python -m pip install --group dev
# Build the package, alternative use `python -m pip install -e .` instead for updates to show up in real time.
python -m pip install .
```

## Stub Generation
Python stubs are automatically generated using `pyo3-stub-gen`. This will update the python folder automatically. If using pip's editable mode, these changes will automatically show up. If not, you will need to reinstall with pip.
```sh
# python -m pip install -e .
cargo run --bin stub_gen
```

```sh
cargo run --bin stub_gen && python -m pip install .
```

## Testing
Our testing uses pytest. This is included as part of the dev group pip install.
```sh
# Testing
python -m pytest
```

### Planned Development

- DMI file modification.
- Better errors and consistent API surface area.
- More improvement of AST walking and code reflection API.
- Pre-defined AST walker superclass with useful behaviors.
- Ability to create new tile definitions in DMM files.
- Passing compiler defines to parser.