from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .drive_label_summary import DriveLabelSummary


@dataclass
class NodeLabel:
    id: str
    node_id: str
    label_id: str
    lifecycle_status: str
    version: int
    label: DriveLabelSummary
    tenant_id: Optional[str] = None
