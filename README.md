# Avulto

[Avulto][] is a Python library for working in the BYOND environment. Its goal is
to provide a straightforward Python API which leverages the [SpacemanDMM][] and
potentially other community libraries.

Its primary use cases are to easily

- read and modify map files
- parse and read icon files
- read the source tree and provide reflection data.

[Avulto]: https://github.com/warriorstar-orion/avulto
[SpacemanDMM]: https://github.com/SpaceManiac/SpacemanDMM

## Usage

Avulto is available as a [release][] on PyPI. See the **Development** section
below for directions on using the library locally.

A Quickstart and the API reference are available at the library's [documentation site][]
and in the `docs/` directory of the repository.

Avulto's API is documented in full in its [stub file][], but the most important
parts of its API are below.

[documentation site]: https://warriorstar-orion.github.io/avulto/
[release]: https://pypi.org/project/avulto/
[stub file]: https://github.com/warriorstar-orion/avulto/blob/main/avulto.pyi

## Development

Avulto is written in Rust and implemented using
[PyO3](https://github.com/PyO3/pyo3), and uses
[maturin](https://www.maturin.rs/) for development. To build and install
locally:

```sh
$ python -m maturin build; python -m pip install .
$ python -m pytest
```

### Planned Development

- More DMI icon data.
- Getting image data directly through SpacemanDMM.
- Better errors and consistent API surface area.
- More reflection data, including method names.

## License

Avulto is licensed under the GPL. See `LICENSE` for more information.

## Acknowledgements

Portions of Avulto are originally based on
[SpacemanDMM](https://github.com/SpaceManiac/SpacemanDMM), copyright Tad
Hardesty and licensed under the GPL.

Portions of Avulto are originally based on
[StrongDMM](https://github.com/SpaiR/StrongDMM), copyright SpaiR and licensed
under the GPL.
