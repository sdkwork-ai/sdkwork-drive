from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .storage_provider import StorageProvider


@dataclass
class StorageProviderBinding:
    id: str
    tenant_id: str
    provider_id: str
    binding_scope: str
    purpose: str
    lifecycle_status: str
    version: int
    storage_provider: StorageProvider
    storage_root_prefix: str
    space_id: Optional[str] = None
