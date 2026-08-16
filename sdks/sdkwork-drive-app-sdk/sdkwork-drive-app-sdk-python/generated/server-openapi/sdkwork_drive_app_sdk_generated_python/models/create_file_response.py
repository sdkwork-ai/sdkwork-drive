from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .drive_node import DriveNode
    from .drive_upload_session import DriveUploadSession


@dataclass
class CreateFileResponse:
    node: DriveNode
    upload_session: DriveUploadSession
