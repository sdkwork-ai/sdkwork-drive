from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateAssetCollectionItemRequest:
    asset_id: str
    sort_order: Optional[int] = None
