Putting It All Together
=======================

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
            typedecl = dme.type_decl(turf)
            icon = typedecl.value('icon')
            icon_state = typedecl.value('icon_state')
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
            turf = tile.find('/turf')[0]
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
