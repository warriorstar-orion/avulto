import os
from typing import Iterator, Any
import pathlib

class Coord3:
    """A three-dimensional integer coordinate. These are currently only returned by certain DMM properties, and are not constructable."""

    x: int
    y: int
    z: int

class SourceLoc:
    """
    Information about the location of a source token in the tree.
    """

    file_path: pathlib.Path
    """The file path of the source location."""
    line: int
    """The line number."""
    column: int
    """The column number."""

class ProcArg:
    """
    A representation of a proc declaration argument.
    """

    arg_name: str
    """The argument name."""
    arg_type: Path | None
    """The argument type, if available."""

class Path:
    """A DM typepath."""

    stem: str
    """The final part of the path."""
    parent: "Path"
    """The parent path."""
    is_root: bool
    """Whether or not the path is `/`."""

    abs: str
    """Returns the *absolute* representation of the path, rooted at `/datum`."""
    rel: str
    """Returns the *relative* or *declared* representation of the path."""

    def __init__(self, value):
        """Returns a new path."""

    def child_of(self, other, strict=False) -> bool:
        """Returns whether the path is a child of `other`.

        If `strict` is true, the current path will not be considered a child of
        itself.
        """

    def parent_of(self, other, strict=False) -> bool:
        """Returns whether the path is a parent of `other`.

        If `strict` is true, the current path will not be considered a parent of
        itself.
        """

    def __truediv__(self, other: str | "Path") -> "Path":
        """Return the path with the specified suffix."""

class Tile:
    """An individual map tile definition."""

    area_path: Path
    """Returns the path of the tile's area. Returns only the first area if multiple exist."""

    turf_path: Path
    """Returns the path of the tile's turf. Returns only the first area if multiple exist."""

    def add_path(self, index, path: Path | str):
        """Add a prefab with the given `path` at `index`."""

    def convert(self) -> list[dict]:
        """Convert the tile definition to a Python data structure.

        The result of `convert` is completely disassociated from the map tile
        definition, and modifying it will have no effect on the original tile.
        """

    def del_prefab(self, index: int):
        """Deletes the prefab at `index`."""

    def del_prefab_var(self, index: int, name: str):
        """Deletes the variable `name` from the prefab at `index`."""

    def find(self, prefix: Path | str, exact=False) -> list[int]:
        """
        Return the indexes of the prefabs prefixed with the given path `prefix`.
        """

    def prefab_path(self, index: int) -> Path:
        """Return the path of the prefab at `index`."""

    def prefab_var(self, index: int, name: str) -> Any:
        """
        Return the value of the property `name` on the prefab at `index`.

        Raises an error if the property does not exist. For a method that
        returns a default if the property does not exist, see get_prefab_var.
        """

    def get_prefab_var(self, index: int, name: str, default: Any = None):
        """
        Returns the value of the property `name` on the prefab at `index`. If
        the property does not exist, return `default`.
        """

    def prefab_vars(self, index: int) -> list[str]:
        """
        Return the list of variable names on the prefab at `index`.
        """

    def set_prefab_var(self, index: int, name: str, val):
        """Set the value of the variable `name` to `val` at `index`."""

    def set_path(self, index: int, path: Path | str):
        """Set the path of the prefab at `index` to `path`."""

class DMM:
    """A DMM file."""

    size: Coord3
    """The number of tiles in each of the map's three dimensions."""
    filepath: pathlib.Path
    """The original filename of the DMM."""

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

class ProcDecl:
    """
    A single proc declaration.
    """

    name: str
    """The name of the proc."""
    type_path: Path
    """The type path the proc is declared on."""
    args: list[ProcArg]
    """The proc arguments."""
    source_loc: SourceLoc
    """The source location of the proc declaration."""

    def walk(self, walker: Any):
        """Walks the proc AST with *walker*, calling any `visit_*` method names on *walker* if they exist for AST node types."""

class VarDecl:
    """
    A single variable declaration.
    """

    name: str
    """The name of the variable."""
    type_path: Path
    """The type path the proc is declared on."""
    declared_type: Path | None
    """The declared type of the variable, if specified."""
    const_val: Any | None
    """The variable's value, if it can be evaluated as a constant expression."""

class TypeDecl:
    """
    A single type declaration.
    """

    path: Path
    """The path of the type."""
    source_loc: SourceLoc
    """The source location of the type declaration."""

    def proc_names(self, declared=False, modified=False, unmodified=False) -> list[str]:
        """Return a list of proc names for the type declaration."""

    def proc_decls(self, name=None) -> list[ProcDecl]:
        """Return proc declarations for the type. If *name* is set, only return proc declarations with this name."""

    def var_names(self, declared=False, modified=False, unmodified=False) -> list[str]:
        """Return a list of variable names for the type declaration."""

    def var_decl(self, name, parents=True) -> VarDecl:
        """Return the proc declaration for variable *name*. If *parents* is True, check up type path if this type does not have this variable set."""

class DME:
    """
    A representation of a single Dreammaker environment.
    """

    filepath: pathlib.Path
    """The original filename of the DMM."""
    types: dict[Path | str, TypeDecl]
    """A mapping of type paths to their declarations."""

    @staticmethod
    def from_file(filename: os.PathLike | str, parse_procs: bool = False) -> "DME":
        """Creates a DME from the given `filename`.

        If parse_procs is True, the entire AST of the codebase is traversed.
        This is slower than the default but provides more reflection
        information.
        """

    def typesof(self, prefix: Path | str) -> list[Path]:
        """Returns a list of type paths with the given `prefix`."""

    def subtypesof(self, prefix: Path | str) -> list[Path]:
        """Returns a list of type paths with the given `prefix`, excluding `prefix` itself."""

class Dir:
    """An enumeration of directions used in icons."""

    NORTH: "Dir"
    SOUTH: "Dir"
    EAST: "Dir"
    WEST: "Dir"
    NORTHEAST: "Dir"
    NORTHWEST: "Dir"
    SOUTHEAST: "Dir"
    SOUTHWEST: "Dir"

class IconState:
    """
    A single icon state in a DMI file.
    """

    name: str
    """The state name."""
    dirs: list[Dir]
    """The directions available in the icon state."""
    dir_count: int
    """The number of directions in the icon state, either 1, 4, or 8."""
    frames: int
    """The number of frames in the icon state."""
    movement: bool
    """Whether or not the state is a movement state."""
    delays: list[float]
    """The icon's frame delays."""
    rewind: bool
    """Whether the icon animates both backwards and forwards."""
    loop_flag: int
    """The number of times the icon animation loops, or 0 for indefinitely."""

    @staticmethod
    def from_data(data: dict[Dir, list[bytes]], width: int = 32, height: int = 32, name: str = "", delays: list[float] | None = None, loops: int = 0, rewind: bool = False, movement: bool = False):
        """
        Creates an IconState with the given frame `data`, which must be a dict of Dirs to arrays of RGBA8 image byte data.
        """

class DMI:
    """
    A DMI file.
    """

    filepath: pathlib.Path
    """The original filename of the DMI."""
    icon_width: int
    """The width of icons in the DMI."""
    icon_height: int
    """The height of icons in the DMI."""
    icon_dims: tuple[int, int]
    """The width and height of icons in the DMI."""
    states: list[IconState]
    """The states in the DMI."""

    @staticmethod
    def new(dims: tuple[int, int]) -> "DMI":
        """
        Creates an empty DMI file with the given icon width-height tuple `dims`.
        """

    @staticmethod
    def from_file(filename: os.PathLike | str) -> "DMI":
        """
        Creates a DMI from the given `filename`.
        """

    def state_names(self) -> list[str]:
        """
        Return a list of strings containing all state names in the file.
        """

    def state(self, name: str) -> IconState:
        """
        Return the icon state with the given `name`. If there are duplicates,
        only the first one is returned. Use `states()` to retrieve duplicates.
        """

    def states(self) -> Iterator[IconState]:
        """
        Iterates over all icon states.
        """

    def data_rgba8(frame: int, dir: Dir) -> bytes:
        """Return the byte data of the spritesheet in 8-bit RGBA."""

class Dmlist:
    """
    A primitive, read-only representation of a DM list. This is used when
    returning constant values of lists from the AST walker, and prefab values
    from the DMM reader. They are not constructable.

    Dmlists only support iterating over keys, and indexing with keys.
    """

    def keys(self) -> Iterator[Any]:
        """
        Iterates over the keys in the list.
        """

    def __getitem__(self, k) -> Any: ...
