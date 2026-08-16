from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class QuotaSummary:
    used_bytes: int
    object_count: int
    tenant_id: Optional[str] = None
    quota_bytes: Optional[int] = None
