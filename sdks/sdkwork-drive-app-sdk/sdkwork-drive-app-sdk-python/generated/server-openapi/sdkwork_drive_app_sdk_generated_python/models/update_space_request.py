from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateSpaceRequest:
    display_name: Optional[str] = None
    presentation_icon: Optional[str] = None
    presentation_color: Optional[str] = None
    description: Optional[str] = None
