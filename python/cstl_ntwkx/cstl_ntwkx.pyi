from ast import Dict, Tuple
from enum import Enum


class ConstellationType(Enum):
    Star = 1
    Delta = 2


class Constellation:
    def add_groundstation(
        self,
        name: str,
        lat: float,
        lon: float,
        alt: float,
    ):
        pass

    def propagate(self, step: int):
        pass


def create_constellation(
    satellites: int,
    planes: int,
    ipc: int,
    altitude: int,
    inclination: float,
    mim_elevation: float,
    constellation_type: ConstellationType
) -> Constellation:
    pass


def extract_graph(constellation: Constellation) -> str:
    pass


def extract_positions_3d(
    constellation: Constellation,
) -> Dict[int, Tuple[float, float, float]]:
    pass


def project_3d_positions(
    constellation: Constellation,
) -> Dict[int, Tuple[float, float, float]]:
    pass
