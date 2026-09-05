from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class QuotaSummary:
    used_bytes: str
    object_count: str
    tenant_id: Optional[str] = None
    quota_bytes: Optional[str] = None
