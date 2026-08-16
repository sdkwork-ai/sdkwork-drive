from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ProviderObjectMutation:
    provider_id: str
    bucket: str
    object_key: str
    changed: bool
