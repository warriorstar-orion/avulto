import os

import pytest

from avulto import DMI, IconState, Dir
from PIL import Image


def get_fixture_path(name):
    return os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures", name)


@pytest.fixture
def dmi() -> DMI:
    return DMI.from_file(get_fixture_path("icon1.dmi"))


def test_dmi_info(dmi: DMI):
    assert dmi.icon_height == 32
    assert dmi.icon_width == 32

    red_circle = dmi.state("red_circle")
    assert not red_circle.movement
    assert red_circle.frames == 1


def test_dmi_creation():
    red = Image.new("RGBA", (32, 32), (255, 0, 0, 255))
    green = Image.new("RGBA", (32, 32), (0, 255, 0, 255))
    yellow = Image.new("RGBA", (32, 32), (255, 255, 0, 255))
    blue = Image.new("RGBA", (32, 32), (0, 0, 255, 255))
    state = IconState.from_data(
        {
            Dir.SOUTH: [red.tobytes()],
            Dir.NORTH: [green.tobytes()],
            Dir.EAST: [blue.tobytes()],
            Dir.WEST: [yellow.tobytes()],
        },
        name="foo",
    )
    dmi = DMI.new((32, 32))
    dmi.states.append(state)
    dmi.save_to("output.dmi")
