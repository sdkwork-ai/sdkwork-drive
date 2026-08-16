from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ProviderBucket:
    provider_id: str
    bucket: str
    exists: bool
