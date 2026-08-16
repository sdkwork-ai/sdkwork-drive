from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class OpenNode:
    id: str
    tenant_id: str
    space_id: str
    node_type: str
    node_name: str
    content_type: Optional[str] = None
    content_length: Optional[int] = None
