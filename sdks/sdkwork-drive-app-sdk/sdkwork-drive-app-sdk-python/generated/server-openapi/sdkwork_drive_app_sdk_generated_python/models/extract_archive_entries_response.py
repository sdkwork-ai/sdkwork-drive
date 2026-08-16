from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .drive_node import DriveNode


@dataclass
class ExtractArchiveEntriesResponse:
    items: List[DriveNode]
    extracted_count: int
