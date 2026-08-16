from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateCommentRequest:
    content: Optional[str] = None
    anchor: Optional[str] = None
    resolved: Optional[bool] = None
