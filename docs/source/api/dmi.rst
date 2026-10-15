``DMI`` --- Icon Parsing
========================

.. autoapiclass:: avulto.DMI

   Instances are created with the following methods:

   .. autoapimethod:: avulto.DMI.from_file
   .. autoapimethod:: avulto.DMI.new

   Once instantiated, the following methods and properties are available:

   .. autoapiproperty:: avulto.DMI.filepath
   .. autoapiproperty:: avulto.DMI.icon_width
   .. autoapiproperty:: avulto.DMI.icon_height
   .. autoapiproperty:: avulto.DMI.icon_dims
   .. autoapiproperty:: avulto.DMI.states

   .. autoapimethod:: avulto.DMI.state_names
   .. autoapimethod:: avulto.DMI.state

   .. autoapimethod:: avulto.DMI.save_to

      `compress` is an value from 0 to 6, corresponding to `oxipng`_
      optimization values.

      .. _oxipng: https://github.com/oxipng/oxipng

.. autoapiclass:: avulto.IconState

   .. autoapimethod:: avulto.IconState.from_data

   .. autoapiproperty:: avulto.IconState.name   
   .. autoapiproperty:: avulto.IconState.frames
   .. autoapiproperty:: avulto.IconState.delay
   .. autoapiproperty:: avulto.IconState.dirs
   .. autoapiproperty:: avulto.IconState.movement
   .. autoapiproperty:: avulto.IconState.rewind
   
   .. autoapimethod:: avulto.IconState.data_rgba8

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
