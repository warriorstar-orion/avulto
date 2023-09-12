import os
from typing import Iterator

class Path:
    """A DM typepath."""

    def child_of(self, other) -> bool:
        """Returns whether the path is a strict child of `other`."""
    def parent_of(self, other) -> bool:
        """Returns whether the path is a strict parent of `other`."""

class Tile:
    """An individual map tile definition."""

    def add_path(self, index, path: Path | str):
        """Add a prefab with the given `path` at `index`."""
    def area_path(self):
        """Returns the path of the tile's area.

        Returns only the first area if multiple exist.
        """
    def convert(self) -> list[dict]:
        """Convert the tile definition to a Python data structure.

        The result of `convert` is completely disassociated from the map tile
        definition, and modifying it will have no effect on the original tile.
        """
    def del_prefab(self, index: int):
        """Deletes the prefab at `index`."""
    def del_prefab_var(self, index: int, name: str):
        """Deletes the variable `name` from the prefab at `index`."""
    def find(self, prefix: str) -> list[int]:
        """
        Return the indexes of the prefabs prefixed with the given path `prefix`.
        """
    def paths(self) -> list[Path]:
        """Returns the paths of all prefabs on the tile."""
    def prefab_path(self, index: int) -> str:
        """Return the path of the prefab at `index`."""
    def prefab_var(self, index: int, name: str) -> any:
        """
        Return the value of the property `name` on the prefab at `index`.
        """
    def prefab_vars(self, index: int) -> list[str]:
        """
        Return the list of variable names on the prefab at `index`.
        """
    def set_prefab_var(self, index: int, name: str, val):
        """Set the value of the variable `name` to `val` at `index`."""
    def set_path(self, index: int, path: Path | str):
        """Set the path of the prefab at `index` to `path`."""
    def turf_path(self):
        """Returns the path of the tile's turf. Returns only the first area if multiple exist."""

class DMM:
    """A DMM file."""

    extents: tuple[int, int, int]
    """The number of tiles in each of the map's three dimensions."""

    @staticmethod
    def from_file(filename: os.PathLike | str) -> "DMM":
        """Creates a DMM from the given `filename`."""
    def coords(self) -> Iterator[tuple[int, int, int]]:
        """Return an iterator over all possible 3D coordinates in the map."""
    def tiles(self) -> Iterator[Tile]:
        """Return an iterator over all unique tiles in the map."""
    def save_to(self, filename: os.PathLike | str):
        """Saves the DMM to the given `filename`."""
    def tiledef(self, x: int, y: int, z: int) -> Tile:
        """Return the tile definition at coords (`x`, `y`, `z`)."""

class TypeDecl:
    """
    A single type declaration.
    """

    def varnames(self) -> list[str]:
        """Return a list of variable names for the type declaration."""
    def value(self, name: str) -> any:
        """Return a Python representation of the variable `name`."""

class DME:
    """
    A representation of a single Dreammaker environment.
    """

    @staticmethod
    def from_file(filename: os.PathLike | str) -> "DME":
        """Creates a DME from the given `filename`."""
    def paths_prefixed(self, prefix: Path | str) -> list[str]:
        """Returns a list of paths with the given `prefix`."""
    def typedecl(self, path: str) -> TypeDecl:
        """Return the type declaration of the given `path`."""

class Dir:
    """An enumeration of directions used in icons."""

class IconState:
    """
    A single icon state in a DMI file.
    """

    def name(self) -> str:
        """The state name."""
    def dirs(self) -> list[Dir]:
        """The directions available in the icon state."""

class DMI:
    """
    A DMI file.
    """

    @staticmethod
    def from_file(filename: str) -> "DMI":
        """
        Creates a DMI from the given `filename`.
        """
    def state_names(self) -> list[str]:
        """
        Return a list of strings containing all state names in the file.
        """
    def state(self, name: str) -> IconState:
        """
        Return the icon state with the given `name`.
        """
