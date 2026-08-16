from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class UpdateStorageProviderRequest:
    name: Optional[str] = None
    endpoint_url: Optional[str] = None
    region: Optional[str] = None
    bucket: Optional[str] = None
    path_style: Optional[bool] = None
    credential_ref: Optional[str] = None
    server_side_encryption_mode: Optional[str] = None
    default_storage_class: Optional[str] = None
    status: Optional[str] = None
    strict_tls: Optional[bool] = None
