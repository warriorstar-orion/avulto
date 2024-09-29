import os

import pytest

from avulto import DME


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"))


def test_dme_typesof(dme: DME):
    assert dme.typesof("/obj/foo") == [
        "/obj/foo",
        "/obj/foo/bar",
        "/obj/foo/baz",
    ]

    assert dme.subtypesof("/obj/foo") == [
        "/obj/foo/bar",
        "/obj/foo/baz",
    ]


def test_missing_type(dme: DME):
    with pytest.raises(RuntimeError) as ex:
        dme.typedecl("/missing_type")

    assert str(ex.value) == "cannot find path /missing_type"


def test_dme_vars(dme: DME):
    foo = dme.typedecl("/obj/foo")
    assert foo.varnames() == ["a", "icon", "icon_state"]
    assert foo.value("a") == 3

    bar = dme.typedecl("/obj/foo/bar")
    assert bar.varnames() == ["a"]
    assert bar.value("a") == 4

    baz = dme.typedecl("/obj/foo/baz")
    assert baz.value("a") == 3


def test_dme_procs(dme: DME):
    foo = dme.typedecl("/obj/foo")
    assert sorted(foo.proc_names()) == ["proc1", "proc2"]
