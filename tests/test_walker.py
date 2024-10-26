import os

import pytest

from avulto import DME


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"), parse_procs=True)


def test_walker_base(dme: DME):
    class VarAndReturnWalker:
        pass

    varw = VarAndReturnWalker()
    dme.walk_proc("/obj/test_object", "var_and_return", varw)
