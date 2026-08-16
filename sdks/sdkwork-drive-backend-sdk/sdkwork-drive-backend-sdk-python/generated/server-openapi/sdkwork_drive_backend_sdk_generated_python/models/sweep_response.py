from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class SweepResponse:
    scanned_count: int
    affected_count: int
    dry_run: bool
