from pathlib import Path

from avulto import DME, DMM

CODE_ROOT = Path("~/ExternalRepos/third_party/Paradise").expanduser()

MAPS = [
    CODE_ROOT / "_maps/map_files/cyberiad/cyberiad.dmm",
    CODE_ROOT / "_maps/map_files/Delta/delta.dmm",
    CODE_ROOT / "_maps/map_files/MetaStation/MetaStation.dmm",
    CODE_ROOT / "_maps/map_files/cerestation/cerestation.dmm",
]

dme = DME.from_file(CODE_ROOT / "paradise.dme")

AIRLOCK_AUTONAMER = "/obj/effect/mapping_helpers/airlock/autoname"
WINDOOR_AUTONAMER = "/obj/effect/mapping_helpers/airlock/windoor/autoname"


def fix_map(filename):
    dmm = DMM.from_file(filename)
    change_made = False

    for tile in dmm.tiles():
        # Tile already has autoname helpers, skip it
        if tile.find(AIRLOCK_AUTONAMER) or tile.find(WINDOOR_AUTONAMER):
            continue

        for airlock in tile.find("/obj/machinery/door/airlock"):
            props = tile.prefab_vars(airlock)
            if "name" in props:
                area_name = dme.type_decl(tile.area_path()).value("name")
                airlock_name = tile.prefab_var(airlock, "name")
                area_name = area_name.replace("\\improper ", "").replace(
                    "\\proper ", ""
                )
                if airlock_name == area_name:
                    tile.del_prefab_var(airlock, "name")
                    tile.add_path(
                        airlock + 1, "/obj/effect/mapping_helpers/airlock/autoname"
                    )
                    change_made = True

        for windoor in tile.find("/obj/machinery/door/window"):
            props = tile.prefab_vars(windoor)
            if "name" in props:
                area_name = dme.type_decl(tile.area_path()).value("name")
                windoor_name = tile.prefab_var(windoor, "name")
                area_name = area_name.replace("\\improper ", "").replace(
                    "\\proper ", ""
                )
                if windoor_name == area_name:
                    tile.del_prefab_var(windoor, "name")
                    tile.add_path(windoor + 1, WINDOOR_AUTONAMER)
                    if "dir" in props:
                        tile.set_prefab_var(
                            windoor + 1, "dir", tile.prefab_var(windoor, "dir")
                        )
                    change_made = True
                elif windoor_name == f"{area_name} Desk":
                    tile.del_prefab_var(windoor, "name")
                    tile.add_path(windoor + 1, WINDOOR_AUTONAMER + "/desk")
                    if "dir" in props:
                        tile.set_prefab_var(
                            windoor + 1, "dir", tile.prefab_var(windoor, "dir")
                        )
                    change_made = True

    if change_made:
        dmm.save_to(filename)


for map in MAPS:
    print(map)
    fix_map(map)
