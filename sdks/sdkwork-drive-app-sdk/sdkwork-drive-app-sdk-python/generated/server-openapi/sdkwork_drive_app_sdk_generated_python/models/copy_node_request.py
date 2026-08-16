from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CopyNodeRequest:
    id: str
    target_space_id: Optional[str] = None
    target_parent_node_id: Optional[str] = None
    node_name: Optional[str] = None
