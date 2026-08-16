from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class ClaimShareLinkResponse:
    share_link_id: str
    node_id: str
    space_id: str
    role: str
    permission_id: str
    already_claimed: bool
