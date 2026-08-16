from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ExtractArchiveEntriesRequest:
    entry_paths: Optional[List[str]] = None
    target_parent_node_id: Optional[str] = None
