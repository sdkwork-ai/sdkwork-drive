from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ProviderObject:
    provider_id: str
    bucket: str
    object_kind: str
    object_key: str
    content_length: int
    content_type: Optional[str] = None
    etag: Optional[str] = None
    version_id: Optional[str] = None
    storage_class: Optional[str] = None
    last_modified_epoch_ms: Optional[int] = None
