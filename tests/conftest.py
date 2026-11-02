from pathlib import Path

import pytest

from avulto import DME, DMI, DMM


def get_fixture_path(name: str) -> str:
    return str(Path(__file__).parent / "fixtures" / name)

@pytest.fixture
def dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"))

@pytest.fixture
def parsed_dme() -> DME:
    return DME.from_file(get_fixture_path("testenv.dme"), parse_procs=True)

@pytest.fixture
def dmm() -> DMM:
    return DMM.from_file(get_fixture_path("map1.dmm"))

@pytest.fixture
def dmi() -> DMI:
    return DMI.from_file(get_fixture_path("icon1.dmi"))
