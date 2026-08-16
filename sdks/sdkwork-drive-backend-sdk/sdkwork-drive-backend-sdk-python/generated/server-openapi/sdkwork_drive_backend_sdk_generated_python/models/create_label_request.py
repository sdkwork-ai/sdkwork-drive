from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateLabelRequest:
    id: str
    label_key: str
    display_name: str
    color: Optional[str] = None
    description: Optional[str] = None
