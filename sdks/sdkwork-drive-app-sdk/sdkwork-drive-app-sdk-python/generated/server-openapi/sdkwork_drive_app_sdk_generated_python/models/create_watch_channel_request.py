from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateWatchChannelRequest:
    id: str
    address: str
    expiration_epoch_ms: int
    space_id: Optional[str] = None
    token: Optional[str] = None
    channel_type: Optional[str] = None
