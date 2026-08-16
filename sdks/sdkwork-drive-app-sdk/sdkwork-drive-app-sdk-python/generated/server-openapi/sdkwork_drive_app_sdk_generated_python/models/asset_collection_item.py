from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AssetCollectionItem:
    id: str
    collection_id: str
    asset_id: str
    tenant_id: Optional[str] = None
    sort_order: Optional[int] = None
