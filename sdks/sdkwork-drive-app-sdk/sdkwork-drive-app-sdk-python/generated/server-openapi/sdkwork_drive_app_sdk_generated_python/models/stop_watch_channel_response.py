from __future__ import annotations
from dataclasses import dataclass
from typing import TYPE_CHECKING, Optional, List, Dict, Any

if TYPE_CHECKING:
    from .drive_watch_channel import DriveWatchChannel


@dataclass
class StopWatchChannelResponse:
    stopped: bool
    channel: DriveWatchChannel
