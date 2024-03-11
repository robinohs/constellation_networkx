from ast import Dict, Tuple
import datetime
from typing import List

import networkx as nx

class Graph:
    pass

class Constellation:
    pass

def create_constellation(
    satellites: int,
    planes: int,
    ipc: int,
    altitude: int,
    inclination: float,
    mim_elevation: float,
) -> Constellation:
    pass

def add_groundstation(
    constellation: Constellation,
    lat: float,
    lon: float,
    alt: float,
) -> Constellation:
    pass

def propagate(constellation: Constellation, step: int) -> Constellation:
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
