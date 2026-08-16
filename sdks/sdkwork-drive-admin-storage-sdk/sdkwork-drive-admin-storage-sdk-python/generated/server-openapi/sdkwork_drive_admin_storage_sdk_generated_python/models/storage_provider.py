from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class StorageProvider:
    id: str
    provider_kind: str
    name: str
    endpoint_url: str
    bucket: str
    path_style: bool
    status: str
    version: int
    credential_configured: bool
    strict_tls: bool
    region: Optional[str] = None
    credential_ref: Optional[str] = None
    server_side_encryption_mode: Optional[str] = None
    default_storage_class: Optional[str] = None
