import os

import pytest

from avulto import DME


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
    with pytest.raises(RuntimeError) as ex:
        dme.type_decl("/missing_type")

    assert str(ex.value) == "cannot find path /missing_type"


def test_dme_vars(dme: DME):
    foo = dme.type_decl("/obj/foo")
    assert foo.var_names() == ["a", "icon", "icon_state"]
    assert foo.var_decl("a").const_val == 3

    bar = dme.type_decl("/obj/foo/bar")
    assert bar.var_names() == ["a"]
    assert bar.var_decl("a").const_val == 4

    baz = dme.type_decl("/obj/foo/baz")
    assert baz.var_decl("a").const_val == 3


def test_dme_procs(dme: DME):
    foo = dme.type_decl("/obj/foo")
    assert sorted(foo.proc_names()) == ["proc1", "proc2"]


def test_proc_decls(dme: DME):
    foo = dme.type_decl("/obj/foo")
    assert [x.name for x in foo.proc_decls("proc1")] == ["proc1"]
