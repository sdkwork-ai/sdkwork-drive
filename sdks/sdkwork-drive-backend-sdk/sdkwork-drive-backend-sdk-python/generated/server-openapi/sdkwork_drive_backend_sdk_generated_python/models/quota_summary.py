from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class QuotaSummary:
    tenant_id: str
    total_bytes: int
    object_count: int
    quota_bytes: Optional[int] = None
