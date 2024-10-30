:class:`DMM` --- Map Processing and Manipulation
================================================

.. class:: DMM

   The :class:`DMM` class provides an extensive API for reading and modifying
   BYOND map files.

   :class:`DMM` instances are created with the following methods:

   .. staticmethod:: from_file(filename: str | os.PathLike[str])

      Read the map file from the *filename* referring to a ".dmm" file.

   Once instantiated, the following properties and methods are available:

   .. property:: extents
      :type: tuple[int, int, int]

      The maximum size of the map's dimensions, i.e. width, length, and height.

   .. property:: max_x
      :type: int

      The maximum X value of the map.

   .. property:: max_y
      :type: int

      The maximum Y value of the map.

   .. property:: max_z
      :type: int

      The maximum Z value of the map.

   .. method:: tiledef(x: int, y: int, z: int) -> Tile

      Returns the :class:`Tile` at the given coordinates.

   .. method:: save_to(filename: str | os.PathLike[str])

      Save the map into the file *filename*.

   .. method:: coords()

      Return an iterator over all possible 3D coordinates.

   .. method:: tiles()

      Return an iterator over all unique :class:`Tile`\s on the map.

   

.. class:: Tile

   :class:`Tile` objects returned from :func:`DMM.tiledef` can be read and
   operated upon with the following methods.

   .. NOTE::

      Methods that modify tile prefabs currently apply to the preset, not the
      individual tile. Future releases will hopefully provide a way to do both.

   .. property:: area_path
      :type: Path

      Returns the tile's area.

   .. property:: turf_path
      :type: Path

      Returns the tile's turf.

   .. method:: convert() -> list[dict]

      Returns a Python representation of all tile prefabs.

   .. method:: find(prefix: str, exact=False) -> list[int]

      Returns the indexes of all the prefabs on the tile with the given
      *prefix*. If *exact* is :const:`True`, then the prefab path must
      match exactly.

   .. method:: only(prefix: str, exact=False) -> int | None

      Returns the index of the only prefab with the given *prefix*, or
      :const:`None` if no such prefab exists. Raises an error if there is more
      than one prefab with the given *prefix*. If *exact* is :const:`True`, then
      the prefab path must match exactly.

   .. method:: prefab_path(index: int) -> Path

      Returns the path of the prefab at *index*.

   .. method:: prefab_vars(index: int) -> list[str]

      Returns the list of var names for the varedits at *index*.

   .. method:: prefab_var(index: int, name: str)

      Returns a Python representation of the value of the var *name* at *index*.

   .. method:: set_prefab_var(index: int, name: str, val)

      Sets the value of the var *name* at *index* to *val*.

   .. method:: set_path(index: int, path: Path | str)

      Sets the path of the prefab at *index* to *path*, preserving any varedits.

   .. method:: add_path(index: int, path: Path | str)

      Adds a prefab with the given *path* at index *index*.

   .. method:: del_prefab(index: int)

      Deletes the prefab at *index*.

   .. method:: del_prefab_var(index: int, name: str)

      Deletes the varedit of the var *name* from the prefab at *index*.
