# Avulto

Avulto is a Python library for working in the BYOND environment. Its goal is to
provide a straightforward Python API which leverages the
[SpacemanDMM](https://github.com/SpaceManiac/SpacemanDMM) and potentially other
community libraries.

## Usage

Avulto has the following API:

### `Path`

`Path` wraps type paths and provides a handful of useful methods.

| Method             | Description                                             |
| ------------------ | ------------------------------------------------------- |
| `parent_of(other)` | Returns whether the path is a strict parent of `other`. |
| `child_of(other)`  | Returns whether the path is a strict child of `other`.  |
| `/`                | Allows easy path suffixing, e.g. `Path("/obj") / "foo"` |

### `DMM`

The `DMM` class allows parsing and manipulation of map files.

| Method                | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `from_file(filename)` | Creates a `DMM` from the given `filename`.             |
| `save_to(filename)`   | Saves the `DMM` to the given `filename`.               |
| `coords()`            | Return an iterator over all possible 3D coordinates.   |
| `tiles()`             | Return an iterator over all unique `Tile`s on the map. |
| `tiledef(x, y, z)`    | Returns the `Tile` at the given coordinates.           |

`Tile` objects returned from `DMM.tiles()` and `DMM.tiledef(x, y, z)` have the
following API:

| Method        | Description                                          |
| ------------- | ---------------------------------------------------- |
| `area_path()` | Returns the tile's area.                             |
| `turf_path()` | Returns the tile's turf.                             |
| `paths()`     | Returns all paths on the tile, in order.             |
| `convert()`   | Provide a Python representation of all tile prefabs. |

Inspecting and manipulating prefabs on a tile is performed on the prefab's index:

| Method                             | Description                                                                |
| ---------------------------------- | -------------------------------------------------------------------------- |
| `find(prefix)`                     | Return the indexes of all the prefabs on the tile with the given `prefix`. |
| `prefab_path(index)`               | Return the path of the prefab at `index`.                                  |
| `prefab_vars(index)`               | Return the list of var names at `index`.                                   |
| `prefab_var(index, name)`          | Return the value of the var `name` at `index`.                             |
| `set_prefab_var(index, name, val)` | Set the value of the var `name` at `index` to `val`.                       |
| `set_path(index, path)`            | Set the path of the prefab at `index` to `path`.                           |
| `add_path(index, path)`            | Add a prefab with the given `path` at `index`.                             |
| `del_prefab(index)`                | Deletes the prefab at `index`.                                             |
| `del_prefab_var(index, name)`      | Deletes the var `name` from the prefab at `index`.                         |

### `DME`

The `DME` class allows parsing DM object code:

| Method                   | Description                                                                       |
| ------------------------ | --------------------------------------------------------------------------------- |
| `from_file(filename)`    | Creates a `DME` from the given `.dme` file.                                       |
| `paths_prefixed(prefix)` | Returns a list of paths with the given `prefix`, including `prefix` if it exists. |
| `typedecl(path)`         | Returns a `TypeDecl` of the object `path`.                                        |

`TypeDecl` objects allow variable inspection:

| Method        | Description                                                |
| ------------- | ---------------------------------------------------------- |
| `varnames()`  | Returns a list of variable names for the type declaration. |
| `value(name)` | Return a Python representation of the variable `name`.     |

### DMI

The `DMI` class allows parsing icon files.

| Method                | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `from_file(filename)` | Creates a `DME` from the given `.dme` file.                      |
| `state_names()`       | Return a list of strings containing all state names in the file. |
| `state(name)`         | Returns the `IconState` with the given `name`.                   |

`IconState` objects allow icon state inspection:

| Method   | Description                                 |
| -------- | ------------------------------------------- |
| `name()` | The icon state name.                        |
| `dirs()` | The directions available in the icon state. |

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
