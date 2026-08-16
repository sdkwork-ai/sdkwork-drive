from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateDownloadGrantRequest:
    requested_ttl_seconds: Optional[int] = None
