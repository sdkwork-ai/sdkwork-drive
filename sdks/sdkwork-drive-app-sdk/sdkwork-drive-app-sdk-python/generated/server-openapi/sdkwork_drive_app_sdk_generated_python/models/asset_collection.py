from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class AssetCollection:
    id: str
    user_id: str
    title: str
    lifecycle_status: str
    created_at: str
    updated_at: str
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    description: Optional[str] = None
    collection_type: Optional[str] = None
    visibility: Optional[str] = None
