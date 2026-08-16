from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class SweepUploadSessionsRequest:
    now_epoch_ms: int
    dry_run: bool
    limit: Optional[int] = None
