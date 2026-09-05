from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class QuotaSummary:
    tenant_id: str
    total_bytes: str
    object_count: str
    quota_bytes: Optional[str] = None
