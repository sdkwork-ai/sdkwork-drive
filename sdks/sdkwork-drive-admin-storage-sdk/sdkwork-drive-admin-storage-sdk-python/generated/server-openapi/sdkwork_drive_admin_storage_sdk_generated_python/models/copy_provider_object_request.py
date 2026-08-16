from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CopyProviderObjectRequest:
    source_object_key: str
    destination_object_key: str
    destination_bucket: Optional[str] = None
    metadata_directive: Optional[str] = None
