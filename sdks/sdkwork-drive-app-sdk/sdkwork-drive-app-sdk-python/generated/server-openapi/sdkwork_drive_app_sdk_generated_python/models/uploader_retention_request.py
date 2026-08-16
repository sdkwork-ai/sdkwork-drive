from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UploaderRetentionRequest:
    mode: str
    ttl_seconds: Optional[int] = None
    cleanup_action: Optional[str] = None
    hard_delete_after_seconds: Optional[int] = None
