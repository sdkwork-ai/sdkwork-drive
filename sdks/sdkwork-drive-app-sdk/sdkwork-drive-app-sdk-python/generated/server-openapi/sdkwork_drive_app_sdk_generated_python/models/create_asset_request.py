from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .media_resource import MediaResource


@dataclass
class CreateAssetRequest:
    drive_node_id: Optional[str] = None
    virtual_reference: Optional[Dict[str, Any]] = None
    title: Optional[str] = None
    description: Optional[str] = None
    scene: Optional[str] = None
    source: Optional[str] = None
    tags: Optional[List[str]] = None
