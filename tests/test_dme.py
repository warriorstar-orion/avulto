import os

import pytest

from avulto import DME, TypePath as p


def get_fixture_path(name: str) -> str:
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
    var_names = [var.name for var in foo.vars.declared() + foo.vars.unmodified()]
    assert all([x in var_names for x in ["a", "icon", "icon_state"]])
    assert foo.vars["a"].const_val == 3

    bar = dme.types["/obj/foo/bar"]
    assert [var.name for var in bar.vars.modified()] == ["a"]
    assert bar.vars["a"].const_val == 4

    baz = dme.types["/obj/foo/baz"]
    assert baz.vars["a"].const_val == 3


def test_dme_procs(dme: DME):
    foo = dme.types["/obj/foo"]
    assert sorted([proc.name for proc in foo.procs.declared()]) == ["proc1", "proc2"]


def test_proc_decls(dme: DME):
    foo = dme.types["/obj/foo"]
    assert [x.name for x in foo.procs["proc1"]] == ["proc1"]


def test_builtin_source_loc(dme: DME):
    db = dme.types["/database"]
    assert str(db.source_loc) == "(builtins):1:1"


def test_root_lookups(dme: DME):
    root = dme.types["/"]
    assert "hell_yeah" in [proc.name for proc in root.procs.declared()]


def test_var_decl_type_path(dme: DME):
    foo = dme.types["/obj/foo"]
    var_decl = foo.vars["a"]
    assert p("/obj/foo") == var_decl.type_path


def test_var_decl_new_call_in_list(dme: DME):
    var_decl = dme.types["/obj/init_list_vardecls"].vars["my_news"]
    assert var_decl.const_val
    news = list(var_decl.const_val.keys())
    assert len(news) == 2

    bar, baz = news
    assert bar.path == p("/datum/foo/bar")
    assert list(bar.args.keys()) == [3, 4]

    assert baz.path == p("/datum/foo/baz")
    assert list(baz.args.keys()) == [10, 12]
