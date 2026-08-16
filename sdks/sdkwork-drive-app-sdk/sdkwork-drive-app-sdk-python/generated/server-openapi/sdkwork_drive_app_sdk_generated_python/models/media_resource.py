from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any


@dataclass
class MediaResource:
    kind: str
    source: str
    id: Optional[str] = None
    url: Optional[str] = None
    public_url: Optional[str] = None
    uri: Optional[str] = None
    object_blob_id: Optional[str] = None
    file_name: Optional[str] = None
    mime_type: Optional[str] = None
    size_bytes: Optional[str] = None
    checksum: Optional[Dict[str, Any]] = None
    width: Optional[int] = None
    height: Optional[int] = None
    duration_seconds: Optional[float] = None
    alt_text: Optional[str] = None
    title: Optional[str] = None
    poster: Optional[MediaResource] = None
    thumbnails: Optional[List[MediaResource]] = None
    variants: Optional[List[MediaResource]] = None
    access: Optional[Dict[str, Any]] = None
    ai: Optional[Dict[str, Any]] = None
    metadata: Optional[Dict[str, Any]] = None
    media_resource_id: Optional[str] = None
    media_type: Optional[str] = None
    content_type: Optional[str] = None
    duration_ms: Optional[int] = None
    checksum_sha256: Optional[str] = None
