# DMM and Tiledefs

## `DMM`

The `DMM` class allows parsing and manipulation of map files.

| Method                | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `from_file(filename)` | Creates a `DMM` from the given `filename`.             |
| `save_to(filename)`   | Saves the `DMM` to the given `filename`.               |
| `coords()`            | Return an iterator over all possible 3D coordinates.   |
| `tiles()`             | Return an iterator over all unique `Tile`s on the map. |
| `tiledef(x, y, z)`    | Returns the `Tile` at the given coordinates.           |
| `extents`             | The maximum size of the map's dimensions.              |

## Tiledefs

`Tile` objects returned from `DMM.tiles()` and `DMM.tiledef(x, y, z)` have the
following API:

| Property    | Description                                          |
| ----------- | ---------------------------------------------------- |
| `area_path` | Returns the tile's area.                             |
| `turf_path` | Returns the tile's turf.                             |

<br />

| Method                     | Description                                                                                   |
| -------------------------- | --------------------------------------------------------------------------------------------- |
| `convert()`                | Provide a Python representation of all tile prefabs.                                          |
| `find(prefix, exact=True)` | Return the indexes of all the prefabs on the tile with the given `prefix`.                    |
| `only(prefix, exact=True)` | Return the index of the only prefab with the given `prefix`. Raises if there's more than one. |

The indexes returned from `find()` and `only()` can be used in the following methods:

| Method                             | Description                                                                |
| ---------------------------------- | -------------------------------------------------------------------------- |
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
