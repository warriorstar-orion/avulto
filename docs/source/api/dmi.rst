:class:`DMI` --- Icon Parsing
=============================

.. class:: DMI

   The :class:`DMI` class provides the ability to create, parse and manipulate
   `.dmi` files.

   :class:`DMI` instances are created with the following methods:

   .. staticmethod:: from_file(filename: str | os.PathLike[str]) -> DMI

      Read the BYOND Icon file from the *filename* referring to a ".dmi" file.

   .. staticmethod:: new(dims: tuple[int, int]) -> DMI

      Create a new DMI expecting states with the given dimensions *dims*.

   Once instantiated, the following methods and properties are available:

   .. property:: filepath
      :type: pathlib.Path

      The original path of the DMI file.

   .. property:: icon_width
      :type: int

      The width of icons in the file.

   .. property:: icon_height
      :type: int

      The height of icons in the file.

   .. property:: icon_dims
      :type: tuple[int, int]

      The width and height of icons in the file.

   .. property:: states
      :type: list[IconState]

      The :class:`IconState`\s in the DMI, in declaration order.

   .. method:: save_to(filename: str | os.PathLike[str], compress: int = 0)

      Save the DMI to a file. `compress` is an value from 0 to 6, corresponding
      to `oxipng`_ optimization values.

      .. _oxipng: https://github.com/oxipng/oxipng

   .. method:: state_names() -> list[str]

      Return a list of all state names in the icon file.

   .. method:: state(name: str) -> IconState

      Returns the :class:`IconState` with the given *name* in the DMI. If
      there are multiple icon states with the same *name*, only the first is
      returned.

Individual icon states are represented by :class:`IconState`.

.. class:: IconState

   .. staticmethod:: from_data(data: dict[Dir, list[bytes]], width: int = 32, height: int = 32, name: str = "", delays: list[float] | None = None, loops: int = 0, rewind: bool = False, movement: bool = False)

      Construct an :class:`IconState` with the given *data* and other arguments.

      *data* must be a dict mapping :class:`Dir`\s to lists of `bytes`
      containing RGBA image data. The number of images in each list must be the
      same, and if there is more than one frame in the icon state, *delays* must
      contain the same number of delay intervals as there are images in the
      state.

   .. property:: name
      :type: str

      The name of the icon state. May not be unique within a file.

   .. property:: frames
      :type: int

      The number of frames in the state.

   .. property:: delays
      :type: list[int]

      A list of delays per frame, in ticks.

   .. property:: dirs
      :type: list[Dir]

      A list of directions available in the icon state.

   .. property:: movement
      :type: bool

      Whether the icon state is a movement state.

   .. property:: rewind
      :type: bool

      Whether the icon state rewinds on animation.

   .. method:: data_rgba8(frame: int, dir: Dir) -> bytes

      Returns the image data for the given 1-indexed frame in RGBA8 bytes.

      Using Pillow_, the image data for a given icon can quickly be turned into
      a :mod:`PIL.Image` object and easily manipulated.

      .. code-block:: python

         from avulto import DMI, Dir
         from PIL import Image
         dmi = DMI.from_file("/SS13/icons/objects/weapons.dmi")
         pistol = dmi.state("pistol")
         data = pistol.data_rgba8(frame=1, dir=Dir.SOUTH)
         image = Image.frombytes("RGBA", size=dmi.icon_dims, data=data)

      .. _Pillow: https://pillow.readthedocs.io/en/stable/
