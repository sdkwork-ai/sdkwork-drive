from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ProviderBucketMutation:
    provider_id: str
    bucket: str
    changed: bool
