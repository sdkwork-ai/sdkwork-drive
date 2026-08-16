from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DriveWatchChannel:
    id: str
    resource_type: str
    channel_type: str
    address: str
    expiration_epoch_ms: int
    lifecycle_status: str
    version: int
    tenant_id: Optional[str] = None
    space_id: Optional[str] = None
    node_id: Optional[str] = None
    resource_id: Optional[str] = None
