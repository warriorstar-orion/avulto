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

def test_procholder_names(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    names = foo.procs.names()

    # Analyze
    assert {"proc1", "proc2"}.issubset(set(names))


def test_procholder_all(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    all_vars = foo.procs.all()

    # Analyze
    assert {"proc1", "proc2"}.issubset(def_to_names(all_vars))


def test_procholder_declared(dme: DME):
    # Setup
    foo = dme.types["/obj/base"]

    # Invoke
    declared = foo.procs.declared()

    # Analyze
    assert {"foobar", "barbaz"} == def_to_names(declared)

def test_procholder_modified(dme: DME):
    # Setup
    bar = dme.types["/obj/base/override"]

    # Invoke
    modified = bar.procs.modified()

    # Analyze
    assert {"foobar"} == def_to_names(modified)


def test_procholder_unmodified(dme: DME):
    # Setup
    bar = dme.types["/obj/base/override"]

    # Invoke
    unmodified = bar.procs.unmodified()

    # Analyze
    assert {"barbaz"}.issubset(def_to_names(unmodified))


def test_procholder_iter(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    names = [var.name for var in foo.procs]

    # Analyze
    assert "proc1" in names


def test_procholder_getitem(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke
    proc1: list[ProcDef] = foo.procs["proc1"]

    # Analyze
    assert len(proc1) == 1
    assert proc1[0].name == "proc1"
    assert proc1[0].type_path == foo.path