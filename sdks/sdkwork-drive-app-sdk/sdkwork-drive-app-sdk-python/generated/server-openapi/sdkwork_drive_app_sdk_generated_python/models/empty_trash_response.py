from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class EmptyTrashResponse:
    deleted_count: str
    skipped_count: str
    has_more: bool
