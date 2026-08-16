from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateCommentRequest:
    id: str
    content: str
    anchor: Optional[str] = None
