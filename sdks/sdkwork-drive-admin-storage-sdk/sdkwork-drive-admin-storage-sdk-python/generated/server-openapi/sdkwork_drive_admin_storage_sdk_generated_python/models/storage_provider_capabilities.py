from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class StorageProviderCapabilities:
    provider_id: str
    provider_kind: str
    supports_multipart_upload: bool
    supports_presigned_upload_part: bool
    supports_presigned_download: bool
    supports_server_side_encryption: bool
    supports_storage_class: bool
    supports_credential_rotation: bool
    supported_server_side_encryption_modes: List[str]
    supported_storage_classes: List[str]
