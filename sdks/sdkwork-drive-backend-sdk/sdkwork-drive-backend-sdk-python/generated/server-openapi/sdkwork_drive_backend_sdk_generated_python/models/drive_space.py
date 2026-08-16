from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveSpace:
    id: str
    tenant_id: str
    owner_subject_type: str
    owner_subject_id: str
    display_name: str
    space_type: str
    lifecycle_status: str
    version: int
