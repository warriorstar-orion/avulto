API Primitives
==============

Several parts of Avulto are useful with all of its APIs.

.. class:: Path


   :class:`Path` wraps type paths and provides several useful methods and
   properties for manipulating and comparing paths::

    >>> from avulto import Path as p

    >>> p('/obj/machinery').parent_of('/obj/machinery/oven')
    True

    >>> p('/turf').child_of('/turf', strict=True)
    False

    >>> p('/obj/machinery') / 'microwave'
    /obj/machinery/microwave

    >>> p('/obj/machinery/microwave').parent
    /obj/machinery

    >>> p('/obj/machinery/microwave').stem
    "microwave"

   .. property:: abs
      :type: str

      Returns the *absolute* representation of the path, rooted at ``/datum``.

      .. code-block:: python

         >>> p('/obj/foo').abs
         '/datum/atom/movable/obj/foo'

         >>> p('/foo').abs
         '/datum/foo'

   .. property:: rel
      :type: str

      Returns the *relative* or *declared* representation of the path.

      .. code-block:: python

         >>> p('/obj/foo').rel
         '/obj/foo'

         >>> p('/datum/foo').rel
         '/datum/foo'

         >>> p('/foo').rel
         '/foo'

   .. property:: parent
      :type: Path

      Returns the immediate parent path of ourselves.

   .. property:: stem
      :type: str

      Returns the last part of our path.

   .. property:: is_root
      :type: bool

      Returns whether the path is ``/``.

   .. method:: parent_of(path: Path | str, strict=False)

      Returns whether we are a parent of *path*. If *strict* is :const:`True`, a
      path will not count as a parent of itself.

   .. method:: child_of(path: Path | str, strict=False)

      Returns whether we are a child of *path*. If *strict* is :const:`True`, a
      path will not count as a child of itself.

.. class:: Coord3

   A three-dimensional integer coordinate. These are currently only returned by
   certain :class:`DMM` properties, and are not constructable.

   .. property:: x
      :type: int

   .. property:: y
      :type: int

   .. property:: z
      :type: int

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
