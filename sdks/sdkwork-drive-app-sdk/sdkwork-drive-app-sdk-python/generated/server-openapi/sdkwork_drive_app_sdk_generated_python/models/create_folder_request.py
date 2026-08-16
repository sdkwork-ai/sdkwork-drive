from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateFolderRequest:
    space_id: str
    node_name: str
    id: Optional[str] = None
    parent_node_id: Optional[str] = None
