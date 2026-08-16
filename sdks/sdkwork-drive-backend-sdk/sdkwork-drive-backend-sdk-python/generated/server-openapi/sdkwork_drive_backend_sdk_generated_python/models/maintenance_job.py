from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class MaintenanceJob:
    id: int
    job_type: str
    status: str
    dry_run: bool
    scanned_count: int
    affected_count: int
    operator_id: str
    started_at: str
    finished_at: str
    created_at: str
    correlation_id: Optional[str] = None
    trace_id: Optional[str] = None
    error_message: Optional[str] = None
