from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class RotateStorageProviderCredentialRequest:
    credential_ref: str
