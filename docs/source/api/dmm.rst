``DMM`` --- Map Processing and Manipulation
===========================================

.. autoapiclass:: avulto.DMM

   The :class:`DMM` class provides an extensive API for reading and modifying
   BYOND map files.

   `DMM` instances are created with the following methods:

   .. autoapimethod:: avulto.DMM.from_file

   Once instantiated, the following properties and methods are available:

   .. autoapiproperty:: avulto.DMM.filepath
   .. autoapiproperty:: avulto.DMM.size

   .. autoapimethod:: avulto.DMM.tiledef
   .. autoapimethod:: avulto.DMM.save_to
   .. autoapimethod:: avulto.DMM.coords
   .. autoapimethod:: avulto.DMM.tiles
   
.. autoapiclass:: avulto.Tile

   :class:`Tile` objects returned from :meth:`DMM.tiledef` or :meth:`DMM.tiles`
   can be read and operated upon with the following methods.

   .. autoapiproperty:: avulto.Tile.area_path
   .. autoapiproperty:: avulto.Tile.turf_path

   .. autoapimethod:: avulto.Tile.convert
   .. autoapimethod:: avulto.Tile.find
   .. autoapimethod:: avulto.Tile.only

   Once the indexes of the prefabs you wish to work with are returned from
   :meth:`Tile.find` or :meth:`Tile.only`, they may be used to operate on the
   prefabs of the tile.

   Prefabs
   -------

   Every Tile is made up of an ordered list of *prefabs*. Each prefab
   corresponds to an atom on that tile's definition, and has a path and optional
   set of varedits.

   Avulto attempts to represent variable edit values with native Python values
   whenever possible. When a varedit is a type path, it is represented with a
   :class:`Path`. When a varedit is a list, it is represented as
   :class:`Dmlist`.

   Prefabs are always accessed via their index, which can be retrieved with
   :func:`Tile.find` or :func:`Tile.only`.

   Reading Prefab Data
   ^^^^^^^^^^^^^^^^^^^

   .. autoapimethod:: avulto.Tile.prefab_path
   .. autoapimethod:: avulto.Tile.prefab_vars
   .. autoapimethod:: avulto.Tile.prefab_var
   .. autoapimethod:: avulto.Tile.get_prefab_var

   Modifying Prefab Data
   ^^^^^^^^^^^^^^^^^^^^^

   .. autoapimethod:: avulto.Tile.set_prefab_var
   .. autoapimethod:: avulto.Tile.set_path
   .. autoapimethod:: avulto.Tile.add_path
   .. autoapimethod:: avulto.Tile.del_prefab
   .. autoapimethod:: avulto.Tile.del_prefab_var
