from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveNode:
    id: str
    space_id: str
    node_type: str
    node_name: str
    lifecycle_status: str
    version: int
    space_type: str
    created_at: str
    updated_at: str
    tenant_id: Optional[str] = None
    parent_node_id: Optional[str] = None
    shortcut_target_node_id: Optional[str] = None
    scene: Optional[str] = None
    source: Optional[str] = None
    content_state: Optional[str] = None
    file_extension: Optional[str] = None
    content_type: Optional[str] = None
    content_type_group: Optional[str] = None
    content_length: Optional[int] = None
    folder_color: Optional[str] = None
