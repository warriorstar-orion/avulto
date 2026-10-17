import os

import pytest

from avulto import DME, SourceLoc
from avulto.ast import Expression, Node, SettingMode


def get_fixture_path(name: str) -> str:
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"), parse_procs=True)


class NodeCollector:
    def __init__(self):
        self.nodes: list[Node] = list()

    def visit_If(self, node: Node.If, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_While(self, node: Node.While, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_DoWhile(self, node: Node.DoWhile, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_ForLoop(self, node: Node.ForLoop, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_ForInfinite(self, node: Node.ForInfinite, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_ForList(self, node: Node.ForList, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_ForRange(self, node: Node.ForRange, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_ForKeyValue(self, node: Node.ForKeyValue, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_Switch(self, node: Node.Switch, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_TryCatch(self, node: Node.TryCatch, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_Var(self, node: Node.Var, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_Setting(self, node: Node.Setting, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_Del(self, node: Node.Del, source_loc: SourceLoc):
        self.nodes.append(node)

    def visit_Crash(self, node: Node.Crash, source_loc: SourceLoc):
        self.nodes.append(node)


def parse_proc(dme: DME, proc_name: str) -> list[Node]:
    collector = NodeCollector()
    dme.types["/obj/node_parse"].procs[proc_name][0].walk(collector)
    return collector.nodes


def test_if_else_if_else(dme: DME):
    node = parse_proc(dme, "if_else_if_else")[0]
    assert isinstance(node, Node.If)
    assert len(node.if_arms) == 2
    assert isinstance(node.if_arms[0][0], Expression.BinaryOp)
    assert len(node.if_arms[0][1]) == 1
    assert node.else_arm
    assert len(node.else_arm) == 1


def test_if_no_else(dme: DME):
    node = parse_proc(dme, "if_no_else")[0]
    assert isinstance(node, Node.If)
    assert len(node.if_arms) == 1
    assert node.else_arm is None


def test_while(dme: DME):
    node = parse_proc(dme, "while_loop")[0]
    assert isinstance(node, Node.While)
    assert isinstance(node.condition, Expression.Identifier)
    assert len(node.block) == 2


def test_do_while(dme: DME):
    node = parse_proc(dme, "do_while_loop")[0]
    assert isinstance(node, Node.DoWhile)
    assert isinstance(node.condition, Expression.Identifier)
    assert len(node.block) == 1


def test_for_loop(dme: DME):
    node = parse_proc(dme, "for_loop")[0]
    assert isinstance(node, Node.ForLoop)
    assert isinstance(node.init, Node.Var)
    assert isinstance(node.test, Expression.BinaryOp)
    assert node.inc is not None
    assert len(node.block) == 1


def test_for_infinite(dme: DME):
    node = parse_proc(dme, "for_infinite")[0]
    assert isinstance(node, Node.ForInfinite)
    assert len(node.block) == 1


def test_for_list(dme: DME):
    node = parse_proc(dme, "for_list")[0]
    assert isinstance(node, Node.ForList)
    assert node.var_type == "/obj"
    assert isinstance(node.in_list, Expression.Identifier)
    assert len(node.block) == 1


def test_for_range(dme: DME):
    node = parse_proc(dme, "for_range")[0]
    assert isinstance(node, Node.ForRange)
    assert isinstance(node.start, Expression.Constant)
    assert isinstance(node.end, Expression.Constant)
    assert isinstance(node.step, Expression.Constant)
    assert len(node.block) == 1


def test_for_key_value(dme: DME):
    node = parse_proc(dme, "for_key_value")[0]
    assert isinstance(node, Node.ForKeyValue)
    assert isinstance(node.in_list, Expression.Identifier)
    assert node.var_type is None
    assert len(node.block) == 1


def test_switch(dme: DME):
    node = parse_proc(dme, "switch_stmt")[0]
    assert isinstance(node, Node.Switch)
    assert len(node.cases) == 2
    assert len(node.cases[0].exact) == 1
    assert len(node.cases[0].range) == 0
    assert len(node.cases[1].exact) == 0
    assert len(node.cases[1].range) == 1
    assert len(node.cases[1].range[0]) == 2
    assert node.default
    assert len(node.default) == 1


def test_switch_no_default(dme: DME):
    node = parse_proc(dme, "switch_no_default")[0]
    assert isinstance(node, Node.Switch)
    assert len(node.cases) == 1
    assert len(node.cases[0].exact) == 2
    assert node.default is None


def test_try_catch(dme: DME):
    node = parse_proc(dme, "try_catch")[0]
    assert isinstance(node, Node.TryCatch)
    assert len(node.try_block) == 1
    assert len(node.catch_params) == 1
    assert len(node.catch_block) == 1


def test_vars(dme: DME):
    nodes = parse_proc(dme, "vars_stmt")
    assert len(nodes) == 2
    assert isinstance(nodes[0], Node.Var)
    assert isinstance(nodes[0].value, Expression.Constant)
    assert isinstance(nodes[1], Node.Var)
    assert nodes[1].value is None


def test_setting(dme: DME):
    node = parse_proc(dme, "setting")[0]
    assert isinstance(node, Node.Setting)
    assert node.mode == SettingMode.ASSIGN
    assert isinstance(node.value, Expression.Constant)


def test_del(dme: DME):
    node = parse_proc(dme, "del_stmt")[0]
    assert isinstance(node, Node.Del)
    assert isinstance(node.expr, Expression.Identifier)


def test_crash(dme: DME):
    node = parse_proc(dme, "crash_stmt")[0]
    assert isinstance(node, Node.Crash)
    assert isinstance(node.expr, Expression.Constant)
