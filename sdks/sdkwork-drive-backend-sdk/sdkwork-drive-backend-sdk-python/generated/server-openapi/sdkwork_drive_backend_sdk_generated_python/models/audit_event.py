from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AuditEvent:
    id: int
    tenant_id: str
    action: str
    resource_type: str
    resource_id: str
    operator_id: str
    created_at: str
    correlation_id: Optional[str] = None
    trace_id: Optional[str] = None
