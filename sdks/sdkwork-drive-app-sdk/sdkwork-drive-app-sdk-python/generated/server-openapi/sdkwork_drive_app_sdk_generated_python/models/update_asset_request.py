from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateAssetRequest:
    title: Optional[str] = None
    description: Optional[str] = None
    scene: Optional[str] = None
    source: Optional[str] = None
    tags: Optional[List[str]] = None
    visibility: Optional[str] = None
