# DMI

The `DMI` class allows parsing icon files.

| Method                | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `from_file(filename)` | Creates a `DME` from the given `.dme` file.                      |
| `state_names()`       | Return a list of strings containing all state names in the file. |
| `state(name)`         | Returns the `IconState` with the given `name`.                   |

<br />

| Property              | Description                                                      |
| --------------------- | ---------------------------------------------------------------- |
| `filepath`            | A pathlib.Path of the original DMI's filename.                   |
| `icon_width`          | The width of icons in the file.                                  |
| `icon_height`         | The height of icons in the file.                                 |

## `IconState`

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
