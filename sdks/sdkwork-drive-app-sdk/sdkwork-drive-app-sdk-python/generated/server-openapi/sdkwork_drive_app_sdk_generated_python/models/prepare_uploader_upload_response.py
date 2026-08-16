from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .uploader_upload_item import UploaderUploadItem
    from .upload_session_mutation_response import UploadSessionMutationResponse


@dataclass
class PrepareUploaderUploadResponse:
    upload_item: UploaderUploadItem
    upload_session: UploadSessionMutationResponse
