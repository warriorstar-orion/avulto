import os

import pytest

from avulto import DME, SourceLoc
from avulto.ast import Expression, Node


def get_fixture_path(name: str) -> str:
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"), parse_procs=True)


def test_walker_base(dme: DME):
    class VarAndReturnWalker:
        def visit_Return(self, node: Node.Return, source_loc: SourceLoc):
            assert str(source_loc.file_path) == "testenv.dm"
            assert source_loc.line == 27
            assert source_loc.column == 2

        def visit_Expr(self, node: Expression, source_loc: SourceLoc):
            pass

        def visit_Call(self, node: Expression.Call, source_loc: SourceLoc):
            pass

        def visit_Identifier(self, node: Expression.Identifier, source_loc: SourceLoc):
            pass

    varw = VarAndReturnWalker()
    dme.types["/obj/test_object"].proc_decls("var_and_return")[0].walk(varw)

def test_visit_call(dme: DME):
    class CallWalker:
        def __init__(self):
            self.calls: list[Expression.Call] = list()

        def visit_Call(self, node: Expression.Call, source_loc: SourceLoc):
            self.calls.append(node)

    walker = CallWalker()
    dme.types["/obj/test_object"].proc_decls("test_visit_call")[0].walk(walker)
    assert len(walker.calls) == 2
    assert all([isinstance(call, Expression.Call) for call in walker.calls])
