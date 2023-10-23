import os

import pytest

from avulto import DMI, Dir, Rect


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dmi() -> DMI:
    return DMI.from_file(get_fixture_path("icon1.dmi"))


def test_dmi_info(dmi: DMI):
    red_circle = dmi.state("red_circle")

    assert not red_circle.movement()
    assert red_circle.frames() == 1
    rect = red_circle.rect(Dir.SOUTH, 0)
    assert rect == Rect(0, 0, 32, 32)
