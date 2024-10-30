Map Parsing and Modification
============================

We'll begin by comparing Avulto's functionality to a tool already used in many
codebases, *UpdatePaths*.

UpdatePaths is a program which takes a set of instructions about the atoms on a
map, then performs some transformation on the map, based on those instructions.
For example, if one wanted to convert all ``/obj/structure/cable`` atoms on a map
to ``/obj/structure/cable/yellow``\s, you could write an UpdatePaths script that looks
like this:

.. code-block:: yaml

    /obj/structure/cable : /obj/structure/cable/yellow{@OLD}


This is fairly straightforward syntax. The ``{@OLD}`` suffix ensures that any
varedits to the cable being repathed remain.

You'd then run this script at the command line like so:

.. code-block:: powershell

    & '.\tools\UpdatePaths\Update Paths.bat' .\tools\UpdatePaths\Scripts\make_cables_yellow.txt

In Avulto, this would look something like this. We'll go over the API shortly,
but this should also be fairly readable on a first pass:

.. code-block:: python

    from pathlib import Path
    from avulto import DMM

    for mapfile in Path("_maps/**/*.dmm"):
        dmm = DMM.from_file(mapfile)
        for tile in dmm.tiles():
            for cable in tile.find('/obj/structure/cable', exact=True):
                tile.set_path(cable, '/obj/structure/cable/yellow')
        dmm.save_to(dmm.filepath)

Now, at first glance, this might seem like a lot more work for no real benefit.
Why write a ten line script when UpdatePaths can do the same thing in one line?

Now let's add another requirement. Let's say we only want to repath cables that
are in areas with a certain subtype. This can't be represented in UpdatePaths.
There's no way to condition a change based on the other contents of a given
tile.

In Avulto, we can do this in the following manner:

.. code-block:: python

    from pathlib import Path
    from avulto import DMM

    for mapfile in Path("_maps/").glob("**/*.dmm"):
        dmm = DMM.from_file(mapfile)
        for tile in dmm.tiles():
            if tile.area_path().child_of('/area/station/science'):
                for cable in tile.find('/obj/structure/cable', exact=True):
                    tile.set_path(cable, '/obj/structure/cable/yellow')
        dmm.save_to(dmm.filepath)


And just like that, we've managed to do something that would have been painful
to do manually, and impossible to do with UpdatePaths.

Note that this isn't to suggest that you should replace UpdatePaths with Avulto
scripts; simply that it is capable of automating tasks in ways that existing
tools may not have the ability to.
