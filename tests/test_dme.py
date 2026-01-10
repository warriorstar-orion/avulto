import os

import pytest

from avulto import DME, Path as p


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"))


def test_dme_typesof(dme: DME):
    foo_types = {
        "/obj/foo",
        "/obj/foo/bar",
        "/obj/foo/baz",
    }
    assert sorted(dme.typesof("/obj/foo")) == sorted(foo_types)
    assert dme.subtypesof("/obj/foo") == [
        "/obj/foo/bar",
        "/obj/foo/baz",
    ]

    datum_subtypes = dme.subtypesof("/datum")
    assert all([x in datum_subtypes for x in foo_types])


def test_missing_type(dme: DME):
    assert "/missing_type" not in dme.types

    with pytest.raises(KeyError) as ex:
        dme.types["/missing_type"]

    assert ex.value.args[0] == "unrecognized path /missing_type"


def test_dme_vars(dme: DME):
    foo = dme.types["/obj/foo"]
    var_names = foo.var_names(declared=True, unmodified=True)
    assert all([x in var_names for x in ["a", "icon", "icon_state"]])
    assert foo.var_decl("a").const_val == 3

    bar = dme.types["/obj/foo/bar"]
    assert bar.var_names(modified=True) == ["a"]
    assert bar.var_decl("a").const_val == 4

    baz = dme.types["/obj/foo/baz"]
    assert baz.var_decl("a").const_val == 3


def test_dme_procs(dme: DME):
    foo = dme.types["/obj/foo"]
    assert sorted(foo.proc_names(declared=True)) == ["proc1", "proc2"]


def test_proc_decls(dme: DME):
    foo = dme.types["/obj/foo"]
    assert [x.name for x in foo.proc_decls("proc1")] == ["proc1"]


def test_builtin_source_loc(dme: DME):
    db = dme.types["/database"]
    assert str(db.source_loc) == "(builtins):1:1"


def test_root_lookups(dme: DME):
    root = dme.types["/"]
    assert "hell_yeah" in root.proc_names(declared=True)


def test_var_decl_type_path(dme: DME):
    foo = dme.types["/obj/foo"]
    var_decl = foo.var_decl("a")
    assert p("/obj/foo") == var_decl.type_path
