from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveLabelSummary:
    id: str
    label_key: str
    display_name: str
    lifecycle_status: str
    version: int
    tenant_id: Optional[str] = None
    color: Optional[str] = None
    description: Optional[str] = None
