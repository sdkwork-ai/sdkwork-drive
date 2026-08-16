from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class NodeCapabilitiesResponse:
    node_id: str
    role: str
    source: str
    permission_id: str
    inherited: bool
    inherited_from_node_id: str
    can_read: bool
    can_comment: bool
    can_write: bool
    can_download: bool
    can_copy: bool
    can_move: bool
    can_trash: bool
    can_restore: bool
    can_delete: bool
    can_share: bool
    can_manage_permissions: bool
    can_manage_versions: bool
    tenant_id: Optional[str] = None
    subject_type: Optional[str] = None
    subject_id: Optional[str] = None
