from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class Change:
    sequence_no: int
    space_id: str
    event_type: str
    actor_id: str
    created_at: str
    tenant_id: Optional[str] = None
    node_id: Optional[str] = None
