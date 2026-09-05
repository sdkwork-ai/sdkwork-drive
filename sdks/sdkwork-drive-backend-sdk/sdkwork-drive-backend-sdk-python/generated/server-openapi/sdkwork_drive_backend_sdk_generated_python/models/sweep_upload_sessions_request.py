from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class SweepUploadSessionsRequest:
    now_epoch_ms: str
    dry_run: bool
    limit: Optional[str] = None
