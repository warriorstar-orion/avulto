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

def test_procholder_names_inherited(dme: DME):
    # Setup
    override = dme.types["/obj/base/override"]

    # Invoke
    names = override.procs.names()

    # Analyze
    assert {"foobar", "barbaz"}.issubset(set(names))

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

def test_procholder_declared_duped(dme: DME):
    # Setup
    foo = dme.types["/obj/test_object_2"]

    # Invoke
    declared = foo.procs.declared()

    # Analyze
    assert len(declared) == 2
    assert declared[0].name == "dupe_named_proc"
    assert declared[1].name == "dupe_named_proc"

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


def test_procholder_getitem_inherited(dme: DME):
    # Setup
    override = dme.types["/obj/base/override"]

    # Invoke
    barbaz: list[ProcDef] = override.procs["barbaz"]

    # Analyze
    assert len(barbaz) == 1
    assert barbaz[0].name == "barbaz"
    assert barbaz[0].type_path == dme.types["/obj/base"].path


def test_procholder_getitem_missing(dme: DME):
    # Setup
    foo = dme.types["/obj/foo"]

    # Invoke / Analyze
    with pytest.raises(KeyError):
        foo.procs["missing_proc"]


def test_procholder_getitem_duped(dme: DME):
    # Setup
    override = dme.types["/obj/test_object_2"]

    # Invoke
    dupe_named_proc: list[ProcDef] = override.procs["dupe_named_proc"]

    # Analyze
    assert len(dupe_named_proc) == 2
    assert dupe_named_proc[0].name == "dupe_named_proc"
    assert dupe_named_proc[1].name == "dupe_named_proc"