from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateShareLinkRequest:
    role: Optional[str] = None
    expires_at_epoch_ms: Optional[int] = None
    download_limit: Optional[int] = None
