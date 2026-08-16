from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateLabelRequest:
    display_name: Optional[str] = None
    color: Optional[str] = None
    description: Optional[str] = None
