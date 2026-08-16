from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AssetRelation:
    id: str
    asset_id: str
    relation_type: str
    lifecycle_status: str
    tenant_id: Optional[str] = None
    related_asset_id: Optional[str] = None
    source_domain: Optional[str] = None
    source_resource_type: Optional[str] = None
    source_resource_id: Optional[str] = None
    metadata: Optional[Dict[str, Any]] = None
