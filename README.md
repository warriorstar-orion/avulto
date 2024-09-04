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

Avulto's API is documented in full in its [stub file][], but the
most important parts of its API are below.

[release]: https://pypi.org/project/avulto/
[stub file]: https://github.com/warriorstar-orion/avulto/blob/main/avulto.pyi

### `Path`

`Path` wraps type paths and provides `parent_of(other, strict=False)` and
`child_of(other, strict=False)` for comparing paths. When `strict`, the current
path does not count as a parent or child of itself.It also supports the division
operator for easy path suffixing.

```py
from avulto import Path as p

>>> p('/obj/machinery').parent_of('/obj/machinery/oven')
True

>>> p('/turf').child_of('/turf', strict=True)
False

>>> p('/obj/machinery') / 'microwave'
/obj/machinery/microwave
```

### `DMM`

The `DMM` class allows parsing and manipulation of map files.

| Method                | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `from_file(filename)` | Creates a `DMM` from the given `filename`.             |
| `save_to(filename)`   | Saves the `DMM` to the given `filename`.               |
| `coords()`            | Return an iterator over all possible 3D coordinates.   |
| `tiles()`             | Return an iterator over all unique `Tile`s on the map. |
| `tiledef(x, y, z)`    | Returns the `Tile` at the given coordinates.           |
| `extents`             | The maximum size of the map's dimensions.              |

`Tile` objects returned from `DMM.tiles()` and `DMM.tiledef(x, y, z)` have the
following API:

| Method        | Description                                          |
| ------------- | ---------------------------------------------------- |
| `area_path()` | Returns the tile's area.                             |
| `turf_path()` | Returns the tile's turf.                             |
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

Note that the mutative functions above currently apply to the preset, not the
individual tile. Future releases will hopefully provide a way to do both.

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
| `procnames()` | Returns a list of proc names for the type declaration.     |
| `varnames()`  | Returns a list of variable names for the type declaration. |
| `value(name)` | Return a Python representation of the variable `name`.     |

### DMI

The `DMI` class allows parsing icon files.

| Method                | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `from_file(filename)` | Creates a `DME` from the given `.dme` file.                      |
| `state_names()`       | Return a list of strings containing all state names in the file. |
| `state(name)`         | Returns the `IconState` with the given `name`.                   |

| Property              | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `filepath`            | A pathlib.Path of the original DMI's filename.                   |
| `icon_width`          | The width of icons in the file.                                  |
| `icon_height`         | The height of icons in the file.                                 |

`IconState` objects allow icon state inspection:

| Property            | Description                                                          |
| ------------------- | -------------------------------------------------------------------- |
| `name`              | The icon state name.                                                 |
| `delays`            | The delay per frame, in ticks.                                       |
| `dirs`              | The directions available in the icon state.                          |
| `frames`            | The number of frames in the state.                                   |
| `movement`          | Whether the icon state is a movement state.                          |
| `rewind`            | Whether the state rewinds on animation.                              |
| `data_rgba8(frame)` | Returns the image data for the given 1-indexed frame in RGBA8 bytes. |

Using [Pillow][], the image data for a given icon can quickly be turned into an
`Image` object and easily manipulated:

```py
>>> from avulto import DMI, Dir
>>> from PIL import Image
>>> dmi = DMI.from_file("/SS13/icons/objects/weapons.dmi")
>>> pistol = dmi.state("pistol")
>>> data = pistol.data_rgba8(frame=1, dir=Dir.SOUTH)
>>> image = Image.frombytes("RGBA", size=(dmi.icon_width, dmi.icon_height), data=data)
```

[Pillow]: https://pillow.readthedocs.io/en/stable/

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
