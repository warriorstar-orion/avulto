import os
from pathlib import Path

import pytest

from avulto import DMM, Path as p


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dmm() -> DMM:
    return DMM.from_file(get_fixture_path("map1.dmm"))


def test_dmm_pathlib():
    assert DMM.from_file(Path(get_fixture_path("map1.dmm")))


def test_dmm_extents(dmm: DMM):
    assert dmm.size == (10, 10, 1)


def test_dmm_get_object(dmm: DMM):
    assert dmm.tiledef(10, 10, 1).find("/obj/foo") == [0]


def test_dmm_get_objvar(dmm: DMM):
    assert dmm.tiledef(7, 7, 1).find("/obj/foo") == [0]
    assert dmm.tiledef(7, 7, 1).find(p("/obj/foo")) == [0]
    assert dmm.tiledef(7, 7, 1).prefab_var(0, "a") == 4


def test_dmm_tile_eq(dmm: DMM):
    assert dmm.tiledef(7, 7, 1) == dmm.tiledef(7, 7, 1)
    assert dmm.tiledef(1, 1, 1) == dmm.tiledef(1, 2, 1)
