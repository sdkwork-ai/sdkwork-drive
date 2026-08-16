from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class SetDefaultStorageProviderBindingRequest:
    provider_id: str
    space_id: Optional[str] = None
    space_type: Optional[str] = None
    storage_root_prefix: Optional[str] = None
