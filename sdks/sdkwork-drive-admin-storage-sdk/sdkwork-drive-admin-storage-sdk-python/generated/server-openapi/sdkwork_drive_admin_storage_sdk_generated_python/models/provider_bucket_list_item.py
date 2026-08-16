from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ProviderBucketListItem:
    bucket: str
    configured: bool
    creation_date_epoch_ms: Optional[int] = None
