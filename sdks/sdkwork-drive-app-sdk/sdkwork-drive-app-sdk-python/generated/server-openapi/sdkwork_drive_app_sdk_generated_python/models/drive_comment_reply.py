from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveCommentReply:
    id: str
    node_id: str
    comment_id: str
    content: str
    lifecycle_status: str
    version: int
    created_by: str
    updated_by: str
    created_at: str
    updated_at: str
    tenant_id: Optional[str] = None
