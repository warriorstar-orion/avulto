Quickstart
==========

By way of introduction, let's start with some example usages of Avulto.

Map Parsing and Modification
****************************

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

Icon Parsing and Extraction
***************************

Now let's move on to ways Avulto can operate on DMI files. A fairly common task
one might find themselves facing is searching a project's DMI files for an icon
with the name "computer". Rather than trying to do this manually, a simple Avulto
script can do the job for us:

.. code-block:: python

    from pathlib import Path
    from avulto import DMI

    for iconfile in Path("icons").glob("**/*.dmi"):
        dmi = DMI.from_file(iconfile)
        if 'computer' in dmi.state_names():
            print(iconfile)

That's all that's necessary to quickly search the entire project for icons with
a given state name.

We can also export image data from icon files, which we will examine later.

Environment Parsing
*******************

Avulto isn't limited to working with map and icon files; it has basic support
for inspecting the codebase itself.

Let's say you are a typical SS13 codebase, and you have a Syndicate antagonist
with an uplink, and it has many uplink items, like so:

.. code-block:: c

	/datum/uplink_item/stealthy_weapons/garrote
		name = "Fiber Wire Garrote"
		item = /obj/item/garrote
		reference = "GAR"
		cost = 30

You want to get a listing of every uplink item, and how much it costs, in one
list. We can get this list like so:

.. code-block:: python

	from pathlib import Path
	from collections import namedtuple
	from avulto import DME

	dme = DME.from_file("paradise.dme")

	for pth in dme.typesof('/datum/uplink_item'):
		typedecl = dme.types[pth]
		name = typedecl.var_decl('name').const_val
		cost = typedecl.var_decl('cost').const_val
		print(f"Name: {name} Cost: {cost}")

And the result will end up looking something like this:

.. code-block::

	Name: Carbine - 40mm Grenade Ammo Box Cost: 20
	Name: Stechkin APS - 10mm Magazine Cost: 10
	Name: Stechkin APS - 10mm Armour Piercing Magazine Cost: 15
	Name: Stechkin APS - 10mm Incendiary Magazine Cost: 15
	Name: Stechkin APS - 10mm Hollow Point Magazine Cost: 20
	Name: Box of Bioterror Syringes Cost: 25
	Name: Bulldog - 12g Buckshot Magazine Cost: 10
	Name: Bulldog - 12g XL Magazine Duffel Bag Cost: 60
	Name: Bulldog - 12g Ammo Duffel Bag Cost: 60
	Name: Bulldog - 12g Dragon's Breath Magazine Cost: 10
	...

Putting It All Together
***********************

Now let's use all three parts of Avulto to perform a cleanup on our maps.

Oftentimes, ``/turf``\s are placed with invalid icon directions, and it is hard to
notice because BYOND will default to an existing default direction. We want to
remove all these invalid direction varedits from our maps. In order to do this,
we need:

1. to be able to inspect each turf's `icon` and `icon_state`, even if they're
   not varedited (access to those values in code);
2. to check the icon file for each state and see what directions it has (ability
   to parse the icon file);
3. to modify the map and remove the invalid `dir` varedits, without removing the
   valid ones (ability to read and make changes to maps).

Since Avulto has access to all this information, we can do this in one script:

.. code-block:: python

    from functools import cache
    from pathlib import Path

    from avulto import DME, DMI, DMM

    dme = DME.from_file("paradise.dme")

    # Simple cache so we don't load icon files repeatedly
    dmi_files = dict()
    known_dirs = dict()
    def get_iconstate_dirs(turf_path):
        if turf_path not in known_dirs:
            typedecl = dme.types[turf]
            icon = typedecl.var_decl('icon').const_val
            icon_state = typedecl.var_decl('icon_state').const_val
            if icon not in dmi_files:
                dmi_files[icon] = DMI.from_file(icon)
            dmi = dmi_files[icon]
            state = dmi.state(icon_state)
            known_dirs[turf_path] = state.dirs()

        return known_dirs[turf_path]

    for mapfile in Path("_maps/").glob("**/*.dmm"):
        dmm = DMM.from_file(mapfile)
        modified = False
        for tile in dmm.tiles():
            turf = tile.only('/turf')
            turf_dir = tile.get_prefab_var(turf, 'dir', Dir.SOUTH)
            if turf_dir not in get_iconstate_dirs(tile.turf_path()):
                modified = True
                tile.del_prefab_var(turf, 'dir')
        if modified:
            dmm.save_to(dmm.filepath)

By combining all of Avulto's APIs, in around 30 lines of code, we've
successfully cleaned up all the turfs on our maps by removing direction varedits
when the turf's icon doesn't include that direction.

Two of the best things about making mapping changes using Avulto is that their
correctness can be checked just by having someone else read the script, and that
they're resistant to merge conflicts since you can just pull down changes from
master and run the script again, without having to think about it.

Conclusion
**********

As you can imagine, having the ability to parse maps, icons, and the codebase at
the same time, with a single library, for use in scripting, can be very
powerful, and enable automation of things that would be challenging to do
manually, or impossible to do with existing tools.
