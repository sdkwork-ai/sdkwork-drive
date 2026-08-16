from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class DownloadPackageItem:
    node_id: str
    node_name: str
    archive_path: str
    bucket: str
    object_key: str
    content_type: str
    content_length: int
    checksum_sha256hex: str
