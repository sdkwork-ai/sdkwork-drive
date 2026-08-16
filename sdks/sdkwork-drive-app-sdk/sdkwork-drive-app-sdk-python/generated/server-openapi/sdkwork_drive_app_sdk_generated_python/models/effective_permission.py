from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class EffectivePermission:
    id: str
    target_node_id: str
    node_id: str
    role: str
    inherited: bool
    inherited_from_node_id: str
    lifecycle_status: str
    version: int
    tenant_id: Optional[str] = None
    subject_type: Optional[str] = None
    subject_id: Optional[str] = None
