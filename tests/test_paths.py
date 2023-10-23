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


def test_ischild():
    assert paths.Area.child_of(paths.Root)


def test_suffix():
    assert p("/obj") / "foo" == p("/obj/foo")
