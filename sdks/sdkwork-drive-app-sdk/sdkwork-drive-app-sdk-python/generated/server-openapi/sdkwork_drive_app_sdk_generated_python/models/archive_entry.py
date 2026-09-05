from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ArchiveEntry:
    path: str
    name: str
    is_directory: bool
    uncompressed_size_bytes: str
    compressed_size_bytes: str
    content_type: Optional[str] = None
