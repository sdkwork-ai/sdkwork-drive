from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveLabel:
    id: str
    tenant_id: str
    label_key: str
    display_name: str
    lifecycle_status: str
    version: int
    color: Optional[str] = None
    description: Optional[str] = None
