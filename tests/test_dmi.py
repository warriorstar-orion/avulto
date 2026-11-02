import os

import pytest
from PIL import Image

from avulto import DMI, Dir, IconState


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
