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

.. autoapiclass:: avulto.SourceLoc

   :class:`SourceLoc` contains source file information about types, procs, and
   vars. This is useful for tasks like linters which want to show line and
   column-level information about a finding.

   .. autoapiproperty:: avulto.SourceLoc.file_path
   .. autoapiproperty:: avulto.SourceLoc.line
   .. autoapiproperty:: avulto.SourceLoc.column

.. autoapiclass:: avulto.Coord3

   .. autoapiproperty:: avulto.Coord3.x
   .. autoapiproperty:: avulto.Coord3.y
   .. autoapiproperty:: avulto.Coord3.z

.. autoapiclass:: avulto.Dmlist

   .. autoapimethod:: avulto.Dmlist.keys

.. autoapiclass:: avulto.Dir

   .. property:: NORTH
   .. property:: SOUTH
   .. property:: EAST
   .. property:: WEST
   .. property:: NORTHEAST
   .. property:: NORTHWEST
   .. property:: SOUTHEAST
   .. property:: SOUTHWEST
