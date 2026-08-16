from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DrivePermission:
    id: str
    node_id: str
    role: str
    inherited: bool
    lifecycle_status: str
    version: int
    tenant_id: Optional[str] = None
    subject_type: Optional[str] = None
    subject_id: Optional[str] = None
