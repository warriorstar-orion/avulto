import os

import pytest

from avulto import DME


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"))


def test_walker_var_and_return(dme: DME):
    class VarAndReturnWalker:
        def visit_Return(self, node):
            raise RuntimeError("feh")
            print(node)
    
    varw = VarAndReturnWalker()
    dme.walk_proc("/obj/test_object", "var_and_return", varw)
    