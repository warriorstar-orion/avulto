import os

import pytest

from avulto import ast, DME


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

def test_visit_call(dme: DME):
    class CallWalker:
        def __init__(self):
            self.calls = list()

        def visit_Call(self, node):
            self.calls.append(node)

    walker = CallWalker()
    dme.walk_proc("/obj/test_object", "test_visit_call", walker)
    assert len(walker.calls) == 2
    assert all([isinstance(call, ast.Expression.Call) for call in walker.calls])
