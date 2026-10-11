import os

import pytest

from avulto import DME, ProcDef, VarDef


def get_fixture_path(name: str) -> str:
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"))

def def_to_names(defs: list[ProcDef]|list[VarDef]) -> set[str]:
    return {element.name for element in defs}

def test_varholder_names(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    names = foo.vars.names()

    # Analyze
    assert {"a", "icon", "icon_state"}.issubset(set(names))


def test_varholder_all(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    all_vars = foo.vars.all()

    # Analyze
    assert {"a", "icon", "icon_state"}.issubset(def_to_names(all_vars))

def test_varholder_all_inherited(dme: DME):
    # Setup
    foo = dme.types["/obj/foo/bar"]

    # Invoke
    all_vars = foo.vars.all()

    # Analyze
    assert {"a", "icon", "icon_state"}.issubset(def_to_names(all_vars))

def test_varholder_declared(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    declared = foo.vars.declared()

    # Analyze
    assert {"a"} == def_to_names(declared)

def test_varholder_modified(dme: DME):
    # Setup
    bar = dme.types["/obj/foo/bar"]

    # Invoke
    modified = bar.vars.modified()

    # Analyze
    assert {"a"} == def_to_names(modified)


def test_varholder_unmodified(dme: DME):
    # Setup
    bar = dme.types["/obj/foo/bar"]

    # Invoke
    unmodified = bar.vars.unmodified()

    # Analyze
    assert {"icon", "icon_state"}.issubset(def_to_names(unmodified))


def test_varholder_iter(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    names = [var.name for var in foo.vars]

    # Analyze
    assert "a" in names


def test_varholder_getitem(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    a_var = foo.vars["a"]

    # Analyze
    assert a_var.name == "a"
    assert a_var.const_val == 3
    assert a_var.type_path == foo.path


def test_varholder_getitem_inherited(dme: DME):
    # Setup
    bar = dme.types["/obj/foo/bar"]

    # Invoke
    inherited = bar.vars["icon"]

    # Analyze
    assert inherited.name == "icon"
    assert inherited.const_val == "icon1.dmi"
    assert inherited.type_path == dme.types["/obj/foo"].path


def test_varholder_getitem_missing(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke / Analyze
    with pytest.raises(KeyError):
        foo.vars["missing_var"]
