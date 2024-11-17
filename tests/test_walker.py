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
        def visit_Return(self, node, source_info):
            print(node, source_info)

    varw = VarAndReturnWalker()
    dme.type_decl("/obj/test_object").proc_decls("var_and_return")[0].walk(varw)

def test_visit_call(dme: DME):
    class CallWalker:
        def __init__(self):
            self.calls = list()

        def visit_Call(self, node, source_info):
            self.calls.append(node)

    walker = CallWalker()
    dme.type_decl("/obj/test_object").proc_decls("test_visit_call")[0].walk(walker)
    assert len(walker.calls) == 2
    assert all([isinstance(call, ast.Expression.Call) for call in walker.calls])
