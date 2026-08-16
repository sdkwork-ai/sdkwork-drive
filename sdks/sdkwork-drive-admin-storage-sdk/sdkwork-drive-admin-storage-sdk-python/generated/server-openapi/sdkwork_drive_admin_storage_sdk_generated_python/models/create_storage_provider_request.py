from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class CreateStorageProviderRequest:
    id: str
    provider_kind: str
    name: str
    endpoint_url: str
    bucket: str
    region: Optional[str] = None
    path_style: Optional[bool] = None
    credential_ref: Optional[str] = None
    server_side_encryption_mode: Optional[str] = None
    default_storage_class: Optional[str] = None
    status: Optional[str] = None
    strict_tls: Optional[bool] = None
