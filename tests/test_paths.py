from avulto import Path as p, paths


def test_paths():
    assert paths.Area == p("/area")
    assert paths.Turf == p("/turf")
    assert paths.Obj == p("/obj")
    assert paths.Mob == p("/mob")
    assert paths.Datum == p("/datum")
    assert paths.Root == p("/")


def test_isparent():
    assert paths.Root.parent_of(p("/datum"))
    assert p("/obj/foo").parent_of(p("/obj/foo/bar"))
    assert p("/obj").parent_of("/obj/foo")


def test_ischild():
    assert paths.Area.child_of(paths.Root)
    assert p("/obj/foo").child_of("/obj")


def test_concat():
    assert paths.Root / "foo" == p("/foo")


def test_suffix():
    assert p("/obj") / "foo" == p("/obj/foo")


def test_parent():
    assert p("/obj").parent == p("/atom/movable")
    assert p("/area/foo").parent == p("/area")


def test_rel_abs():
    assert p("/obj/foo").rel == "/obj/foo"
    assert p("/obj/foo").abs == "/datum/atom/movable/obj/foo"

    assert p("/matrix").abs == "/matrix"

    assert p("/foo").rel == "/foo"
    assert p("/foo").abs == "/datum/foo"

    # Just like BYOND, we care about declaration format
    assert p("/datum/foo").rel == "/datum/foo"
    assert p("/datum/foo").abs == "/datum/foo"


def test_str_compare():
    assert p("/obj/foo") == "/obj/foo"
    assert p("/datum/atom/movable/obj/foo") == "/obj/foo"
    assert "/obj/foo" == p("/obj/foo")
