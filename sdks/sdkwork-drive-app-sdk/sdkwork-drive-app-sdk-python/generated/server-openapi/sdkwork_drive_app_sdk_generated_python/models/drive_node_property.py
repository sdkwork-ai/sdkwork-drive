from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveNodeProperty:
    id: str
    node_id: str
    property_key: str
    property_value: str
    visibility: str
    lifecycle_status: str
    version: int
    tenant_id: Optional[str] = None
