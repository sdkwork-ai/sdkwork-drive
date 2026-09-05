from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class SweepResponse:
    scanned_count: str
    affected_count: str
    dry_run: bool
