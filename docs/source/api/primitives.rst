API Primitives
==============

Several parts of Avulto are useful with all of its APIs.

.. autoapiclass:: avulto.TypePath

   `TypePath` wraps type paths and provides several useful methods and
   properties for manipulating and comparing paths.

   .. autoapiproperty:: avulto.TypePath.abs

      .. code-block:: python

         >>> p('/obj/foo').abs
         '/datum/atom/movable/obj/foo'

         >>> p('/foo').abs
         '/datum/foo'

   .. autoapiproperty:: avulto.TypePath.rel

      .. code-block:: python

         >>> p('/obj/foo').rel
         '/obj/foo'

         >>> p('/datum/foo').rel
         '/datum/foo'

         >>> p('/foo').rel
         '/foo'

   .. autoapiproperty:: avulto.TypePath.parent
   .. autoapiproperty:: avulto.TypePath.stem
   .. autoapiproperty:: avulto.TypePath.is_root
      
   .. autoapimethod:: avulto.TypePath.parent_of
   .. autoapimethod:: avulto.TypePath.child_of

.. class:: SourceLoc

   :class:`SourceLoc` contains source file information about types, procs, and
   vars. This is useful for tasks like linters which want to show line and
   column-level information about a finding.

   .. property:: file_path
      :type: pathlib.Path

      The file path of the source location.

   .. property:: line
      :type: int

      The line number.

   .. property:: column
      :type: int

      The column number.

.. class:: Coord3

   A three-dimensional integer coordinate. These are currently only returned by
   certain :class:`DMM` properties, and are not constructable.

   .. property:: x
      :type: int

   .. property:: y
      :type: int

   .. property:: z
      :type: int

.. class:: Dmlist

   A primitive, read-only representation of a DM list. This is used when
   returning constant values of lists from the AST walker, and prefab values
   from the DMM reader. They are not constructable.

   Dmlists only support iterating over keys, and indexing with keys.

   .. method:: keys()

      Returns an interator over the list's keys.

.. class:: Dir

   Representation of BYOND directions.

   .. property:: NORTH
   .. property:: SOUTH
   .. property:: EAST
   .. property:: WEST
   .. property:: NORTHEAST
   .. property:: NORTHWEST
   .. property:: SOUTHEAST
   .. property:: SOUTHWEST
