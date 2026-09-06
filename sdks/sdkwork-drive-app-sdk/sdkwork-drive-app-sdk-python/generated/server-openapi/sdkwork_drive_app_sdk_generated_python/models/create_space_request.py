from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateSpaceRequest:
    id: str
    display_name: str
    space_type: str
    owner_subject_type: Optional[str] = None
    owner_subject_id: Optional[str] = None
    presentation_icon: Optional[str] = None
    presentation_color: Optional[str] = None
    description: Optional[str] = None
