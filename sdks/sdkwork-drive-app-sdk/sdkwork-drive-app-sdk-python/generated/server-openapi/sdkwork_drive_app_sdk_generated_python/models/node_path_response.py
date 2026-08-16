from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .drive_node import DriveNode


@dataclass
class NodePathResponse:
    items: List[DriveNode]
    path_segments: List[str]
