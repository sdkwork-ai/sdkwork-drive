from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateDownloadUrlResponse:
    download_url: str
    signed_source_url: str
    expires_at_epoch_ms: int
    method: str
