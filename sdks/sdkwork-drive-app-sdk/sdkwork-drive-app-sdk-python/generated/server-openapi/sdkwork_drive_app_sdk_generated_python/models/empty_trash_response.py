from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class EmptyTrashResponse:
    deleted_count: int
    skipped_count: int
    has_more: bool
