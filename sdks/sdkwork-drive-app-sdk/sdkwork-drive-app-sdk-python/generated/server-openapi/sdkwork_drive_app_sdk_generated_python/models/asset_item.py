from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .media_resource import MediaResource


@dataclass
class AssetItem:
    asset_id: str
    drive_space_id: str
    drive_node_id: str
    node_type: str
    asset_kind: str
    title: str
    lifecycle_status: str
    created_at: str
    updated_at: str
    id: Optional[str] = None
    tenant_id: Optional[str] = None
    organization_id: Optional[str] = None
    user_id: Optional[str] = None
    drive_uri: Optional[str] = None
    asset_type: Optional[str] = None
    description: Optional[str] = None
    scene: Optional[str] = None
    source: Optional[str] = None
    source_type: Optional[str] = None
    source_domain: Optional[str] = None
    source_resource_type: Optional[str] = None
    source_resource_id: Optional[str] = None
    tags: Optional[List[str]] = None
    visibility: Optional[str] = None
    resource_snapshot: Optional[MediaResource] = None
    thumbnail_drive_node_id: Optional[str] = None
