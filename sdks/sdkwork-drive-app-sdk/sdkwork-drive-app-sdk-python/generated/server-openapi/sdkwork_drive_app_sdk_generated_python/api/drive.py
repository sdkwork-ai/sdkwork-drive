from typing import Any, Dict, List, Optional
from ..http_client import HttpClient
from ..models import ActivateWebsiteGenerationRequest, ApplyNodeLabelRequest, ArchiveEntryListHttpResponse, AssetActionRequest, AssetItemHttpResponse, AssetListHttpResponse, ChangeListHttpResponse, CheckFavoriteNodesRequest, ClaimShareLinkHttpResponse, CompleteUploadSessionRequest, CopyNodeRequest, CreateAssetRequest, CreateCommentReplyRequest, CreateCommentRequest, CreateDownloadGrantRequest, CreateDownloadPackageRequest, CreateDownloadUrlHttpResponse, CreateDownloadUrlRequest, CreateDriveSandboxDirectoryRequest, CreateDriveSandboxFileRequest, CreateFileHttpResponse, CreateFileRequest, CreateFolderRequest, CreatePermissionRequest, CreateShareLinkHttpResponse, CreateShareLinkRequest, CreateShortcutRequest, CreateSpaceRequest, CreateUploadSessionRequest, CreateWatchChannelRequest, CreateWebsiteRootRequest, CreateWebsiteSyncRequest, DownloadPackageHttpResponse, DriveCommentHttpResponse, DriveCommentListHttpResponse, DriveCommentReplyHttpResponse, DriveCommentReplyListHttpResponse, DriveNodeHttpResponse, DriveNodeListHttpResponse, DriveNodePropertyHttpResponse, DriveNodePropertyListHttpResponse, DrivePermissionHttpResponse, DrivePermissionListHttpResponse, DriveSandboxEntryHttpResponse, DriveSandboxEntryListHttpResponse, DriveSandboxFileContentHttpResponse, DriveSandboxMutationCommandHttpResponse, DriveSandboxVolumeListHttpResponse, DriveSpaceHttpResponse, DriveSpaceListHttpResponse, DriveUploadSessionHttpResponse, DriveWatchChannelHttpResponse, DriveWatchChannelListHttpResponse, EffectivePermissionListHttpResponse, EmptyTrashHttpResponse, EmptyTrashRequest, ExtractArchiveEntriesHttpResponse, ExtractArchiveEntriesRequest, FavoriteNodeHttpResponse, FavoriteNodeRequest, FileVersionHttpResponse, FileVersionListHttpResponse, MarkUploaderPartUploadedRequest, MoveNodeRequest, NodeCapabilitiesHttpResponse, NodeCommandRequest, NodeLabelHttpResponse, NodeLabelListHttpResponse, NodePathHttpResponse, PrepareUploaderUploadHttpResponse, PrepareUploaderUploadRequest, PresignedUploadPartHttpResponse, PresignUploadPartRequest, PurgeDriveSandboxEntryRequest, QuotaSummaryHttpResponse, SdkWorkApiResponse, SetNodePropertyRequest, ShareLinkHttpResponse, ShareLinkListHttpResponse, StartPageTokenHttpResponse, StopWatchChannelHttpResponse, StopWatchChannelRequest, UpdateAssetRequest, UpdateCommentReplyRequest, UpdateCommentRequest, UpdateDriveSandboxEntryRequest, UpdateDriveSandboxFileContentRequest, UpdateNodeRequest, UpdatePermissionRequest, UpdateShareLinkRequest, UpdateSpaceRequest, UploaderUploadPartHttpResponse, WebsiteGenerationActivationHttpResponse, WebsiteRootHttpResponse, WebsiteRootListHttpResponse, WebsiteSyncActivationHttpResponse, WebsiteSyncHttpResponse, WebsiteSyncVersionRequest

def _append_query_string(path: str, raw_query_string: str) -> str:
    query = raw_query_string.lstrip('?')
    if not query:
        return path
    separator = '&' if '?' in path else '?'
    return f"{path}{separator}{query}"

def serialize_path_parameter(value: Any, spec: Dict[str, Any]) -> str:
    if value is None:
        return ''

    style = str(spec.get('style') or 'simple')
    name = str(spec.get('name') or '')
    explode = bool(spec.get('explode'))
    if isinstance(value, (list, tuple)):
        return serialize_path_array(name, value, style, explode)
    if isinstance(value, dict):
        return serialize_path_object(name, value, style, explode)
    return path_prefix(name, style) + encode_path_value(serialize_path_primitive(value))


def serialize_path_array(name: str, values: Any, style: str, explode: bool) -> str:
    serialized = [encode_path_value(serialize_path_primitive(item)) for item in values if item is not None]
    if not serialized:
        return path_prefix(name, style)
    if style == 'matrix':
        return ''.join(f";{name}={item}" for item in serialized) if explode else f";{name}={','.join(serialized)}"
    return path_prefix(name, style) + ('.' if explode else ',').join(serialized)


def serialize_path_object(name: str, value: Dict[str, Any], style: str, explode: bool) -> str:
    entries = [(key, entry_value) for key, entry_value in value.items() if entry_value is not None]
    if not entries:
        return path_prefix(name, style)
    if style == 'matrix':
        if explode:
            return ''.join(f";{encode_path_value(str(key))}={encode_path_value(serialize_path_primitive(entry_value))}" for key, entry_value in entries)
        serialized = ','.join(item for key, entry_value in entries for item in (encode_path_value(str(key)), encode_path_value(serialize_path_primitive(entry_value))))
        return f";{name}={serialized}"
    if explode:
        separator = '.' if style == 'label' else ','
        serialized = separator.join(f"{encode_path_value(str(key))}={encode_path_value(serialize_path_primitive(entry_value))}" for key, entry_value in entries)
    else:
        serialized = ','.join(item for key, entry_value in entries for item in (encode_path_value(str(key)), encode_path_value(serialize_path_primitive(entry_value))))
    return path_prefix(name, style) + serialized


def path_prefix(name: str, style: str) -> str:
    if style == 'label':
        return '.'
    if style == 'matrix':
        return f";{name}"
    return ''


def encode_path_value(value: str) -> str:
    from urllib.parse import quote

    return quote(value, safe='')


def serialize_path_primitive(value: Any) -> str:
    if isinstance(value, dict):
        import json

        return json.dumps(value, separators=(',', ':'))
    return str(value)


def build_query_string(parameters: List[Dict[str, Any]]) -> str:
    pairs: List[str] = []
    for parameter in parameters:
        append_serialized_parameter(pairs, parameter)
    return '&'.join(pairs)


def append_serialized_parameter(pairs: List[str], parameter: Dict[str, Any]) -> None:
    value = parameter.get('value')
    if value is None:
        return

    name = str(parameter.get('name') or '')
    allow_reserved = bool(parameter.get('allow_reserved'))
    content_type = parameter.get('content_type')
    if content_type:
        import json

        pairs.append(f"{encode_query_component(name)}={encode_query_value(json.dumps(value, separators=(',', ':')), allow_reserved)}")
        return

    style = str(parameter.get('style') or 'form')
    explode = bool(parameter.get('explode'))
    if style == 'deepObject':
        append_deep_object_parameter(pairs, name, value, allow_reserved)
        return
    if isinstance(value, (list, tuple)):
        append_array_parameter(pairs, name, value, style, explode, allow_reserved)
        return
    if isinstance(value, dict):
        append_object_parameter(pairs, name, value, style, explode, allow_reserved)
        return

    pairs.append(f"{encode_query_component(name)}={encode_query_value(serialize_primitive(value), allow_reserved)}")


def append_array_parameter(
    pairs: List[str],
    name: str,
    value: Any,
    style: str,
    explode: bool,
    allow_reserved: bool,
) -> None:
    values = [serialize_primitive(item) for item in value if item is not None]
    if not values:
        return

    if style == 'form' and explode:
        for item in values:
            pairs.append(f"{encode_query_component(name)}={encode_query_value(item, allow_reserved)}")
        return

    pairs.append(f"{encode_query_component(name)}={encode_query_value(','.join(values), allow_reserved)}")


def append_object_parameter(
    pairs: List[str],
    name: str,
    value: Dict[str, Any],
    style: str,
    explode: bool,
    allow_reserved: bool,
) -> None:
    entries = [(key, entry_value) for key, entry_value in value.items() if entry_value is not None]
    if not entries:
        return

    if style == 'form' and explode:
        for key, entry_value in entries:
            pairs.append(f"{encode_query_component(str(key))}={encode_query_value(serialize_primitive(entry_value), allow_reserved)}")
        return

    serialized = ','.join(
        item
        for key, entry_value in entries
        for item in (str(key), serialize_primitive(entry_value))
    )
    pairs.append(f"{encode_query_component(name)}={encode_query_value(serialized, allow_reserved)}")


def append_deep_object_parameter(pairs: List[str], name: str, value: Any, allow_reserved: bool) -> None:
    if not isinstance(value, dict):
        pairs.append(f"{encode_query_component(name)}={encode_query_value(serialize_primitive(value), allow_reserved)}")
        return

    for key, entry_value in value.items():
        if entry_value is None:
            continue
        pairs.append(f"{encode_query_component(f'{name}[{key}]')}={encode_query_value(serialize_primitive(entry_value), allow_reserved)}")


def serialize_primitive(value: Any) -> str:
    if isinstance(value, dict):
        import json

        return json.dumps(value, separators=(',', ':'))
    return str(value)


def encode_query_component(value: str) -> str:
    from urllib.parse import quote

    return quote(value, safe='')


def encode_query_value(value: str, allow_reserved: bool) -> str:
    from urllib.parse import quote

    return quote(value, safe=':/?#[]@!$&\'()*+,;=' if allow_reserved else '')

def build_request_headers(headers: Dict[str, Dict[str, Any]], cookies: Optional[Dict[str, Dict[str, Any]]] = None) -> Optional[Dict[str, str]]:
    request_headers: Dict[str, str] = {}
    for name, parameter in headers.items():
        serialized = serialize_parameter_value(parameter)
        if serialized is not None:
            request_headers[name] = serialized

    cookie_header = build_cookie_header(cookies or {})
    if cookie_header:
        request_headers['Cookie'] = (
            f"{request_headers['Cookie']}; {cookie_header}"
            if 'Cookie' in request_headers
            else cookie_header
        )

    return request_headers or None


def build_cookie_header(cookies: Dict[str, Dict[str, Any]]) -> Optional[str]:
    from urllib.parse import quote

    pairs: List[str] = []
    for name, parameter in cookies.items():
        serialized = serialize_parameter_value(parameter)
        if serialized is not None:
            pairs.append(f"{quote(str(name), safe='')}={quote(serialized, safe='')}")
    return '; '.join(pairs) if pairs else None


def serialize_parameter_value(parameter: Optional[Dict[str, Any]]) -> Optional[str]:
    value = None if parameter is None else parameter.get('value')
    if value is None:
        return None
    if parameter and parameter.get('content_type'):
        import json

        return json.dumps(value, separators=(',', ':'))
    if isinstance(value, (list, tuple)):
        return ','.join(serialize_header_primitive(item) for item in value if item is not None)
    if isinstance(value, dict):
        return serialize_header_object(value, bool(parameter and parameter.get('explode')))
    return serialize_header_primitive(value)


def serialize_header_object(value: Dict[str, Any], explode: bool) -> str:
    entries = [(key, entry_value) for key, entry_value in value.items() if entry_value is not None]
    if explode:
        return ','.join(f"{key}={serialize_header_primitive(entry_value)}" for key, entry_value in entries)
    return ','.join(item for key, entry_value in entries for item in (str(key), serialize_header_primitive(entry_value)))


def serialize_header_primitive(value: Any) -> str:
    return str(value)


class DriveApi:
    """drive drive API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.changes = DriveChangesApi(client)
        self.download_tokens = DriveDownloadTokensApi(client)
        self.download_urls = DriveDownloadUrlsApi(client)
        self.favorites = DriveFavoritesApi(client)
        self.quotas = DriveQuotasApi(client)
        self.nodes = DriveNodesApi(client)
        self.comments = DriveCommentsApi(client)
        self.comment_replies = DriveCommentRepliesApi(client)
        self.download_grants = DriveDownloadGrantsApi(client)
        self.node_labels = DriveNodeLabelsApi(client)
        self.permissions = DrivePermissionsApi(client)
        self.node_properties = DriveNodePropertiesApi(client)
        self.share_links = DriveShareLinksApi(client)
        self.trash = DriveTrashApi(client)
        self.versions = DriveVersionsApi(client)
        self.property_nodes = DrivePropertyNodesApi(client)
        self.recent = DriveRecentApi(client)
        self.search = DriveSearchApi(client)
        self.shared_with_me = DriveSharedWithMeApi(client)
        self.sandboxes = DriveSandboxesApi(client)
        self.sandbox_entries = DriveSandboxEntriesApi(client)
        self.sandbox_directories = DriveSandboxDirectoriesApi(client)
        self.sandbox_files = DriveSandboxFilesApi(client)
        self.sandbox_file_contents = DriveSandboxFileContentsApi(client)
        self.spaces = DriveSpacesApi(client)
        self.website_roots = DriveWebsiteRootsApi(client)
        self.move_destinations = DriveMoveDestinationsApi(client)
        self.upload_sessions = DriveUploadSessionsApi(client)
        self.watch_channels = DriveWatchChannelsApi(client)
        self.download_packages = DriveDownloadPackagesApi(client)
        self.archive_entries = DriveArchiveEntriesApi(client)
        self.uploader = DriveUploaderApi(client)
        self.assets = DriveAssetsApi(client)


class DriveChangesApi:
    """drive drive.changes API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.start_page_token = DriveChangesStartPageTokenApi(client)


    def list(self, space_id: str, cursor: Optional[int] = None, page_size: Optional[int] = None) -> ChangeListHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/changes", query))

    def watch(self, body: CreateWatchChannelRequest) -> DriveWatchChannelHttpResponse:
        """Create a push notification channel for Drive changes"""
        return self._client.post(f"/app/v3/api/drive/changes/watch", json=body)

class DriveChangesStartPageTokenApi:
    """drive drive.changes.start_page_token API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, space_id: str) -> StartPageTokenHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/changes/start_page_token", query))

class DriveDownloadTokensApi:
    """drive drive.download_tokens API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, token: str) -> CreateDownloadUrlHttpResponse:
        return self._client.get(f"/app/v3/api/drive/download_tokens/{serialize_path_parameter(token, {'name': 'token', 'style': 'simple', 'explode': False})}")

class DriveDownloadUrlsApi:
    """drive drive.download_urls API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, body: CreateDownloadUrlRequest) -> CreateDownloadUrlHttpResponse:
        return self._client.post(f"/app/v3/api/drive/download_urls", json=body)

class DriveFavoritesApi:
    """drive drive.favorites API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, space_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None, sort_by: Optional[str] = None, sort_order: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortBy', 'value': sort_by, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortOrder', 'value': sort_order, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/favorites", query))

    def check(self, body: CheckFavoriteNodesRequest) -> SdkWorkApiResponse:
        return self._client.post(f"/app/v3/api/drive/favorites/check", json=body)

    def update(self, node_id: str, body: FavoriteNodeRequest) -> FavoriteNodeHttpResponse:
        return self._client.put(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/favorite", json=body)

    def delete(self, node_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/favorite")

class DriveQuotasApi:
    """drive drive.quotas API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self) -> QuotaSummaryHttpResponse:
        return self._client.get(f"/app/v3/api/drive/quotas/summary")

class DriveNodesApi:
    """drive drive.nodes API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.capabilities = DriveNodesCapabilitiesApi(client)
        self.download_urls = DriveNodesDownloadUrlsApi(client)
        self.path = DriveNodesPathApi(client)
        self.files = DriveNodesFilesApi(client)
        self.folders = DriveNodesFoldersApi(client)
        self.shortcuts = DriveNodesShortcutsApi(client)


    def update(self, node_id: str, body: UpdateNodeRequest) -> DriveNodeHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}", json=body)

    def retrieve(self, node_id: str) -> DriveNodeHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}")

    def delete(self, node_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}")

    def copy(self, node_id: str, body: CopyNodeRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/copy", json=body)

    def move(self, node_id: str, body: MoveNodeRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/move", json=body)

    def list(self, space_id: str, parent_node_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None, sort_by: Optional[str] = None, sort_order: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'parentNodeId', 'value': parent_node_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortBy', 'value': sort_by, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortOrder', 'value': sort_order, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}/nodes", query))

    def watch(self, node_id: str, body: CreateWatchChannelRequest) -> DriveWatchChannelHttpResponse:
        """Create a push notification channel for a Drive node"""
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/watch", json=body)

class DriveNodesCapabilitiesApi:
    """drive drive.nodes.capabilities API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str) -> NodeCapabilitiesHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/capabilities")

class DriveNodesDownloadUrlsApi:
    """drive drive.nodes.download_urls API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, node_id: str, requested_ttl_seconds: Optional[int] = None) -> CreateDownloadUrlHttpResponse:
        query = build_query_string([
            {'name': 'requestedTtlSeconds', 'value': requested_ttl_seconds, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/download_url", query))

class DriveNodesPathApi:
    """drive drive.nodes.path API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, node_id: str) -> NodePathHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/path")

class DriveNodesFilesApi:
    """drive drive.nodes.files API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, body: CreateFileRequest) -> CreateFileHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/files", json=body)

class DriveNodesFoldersApi:
    """drive drive.nodes.folders API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, body: CreateFolderRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/folders", json=body)

class DriveNodesShortcutsApi:
    """drive drive.nodes.shortcuts API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, body: CreateShortcutRequest) -> DriveNodeHttpResponse:
        """Create a shortcut node"""
        return self._client.post(f"/app/v3/api/drive/nodes/shortcuts", json=body)

class DriveCommentsApi:
    """drive drive.comments API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveCommentListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments", query))

    def create(self, node_id: str, body: CreateCommentRequest) -> DriveCommentHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments", json=body)

    def retrieve(self, node_id: str, comment_id: str) -> DriveCommentHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}")

    def update(self, node_id: str, comment_id: str, body: UpdateCommentRequest) -> DriveCommentHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, node_id: str, comment_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}")

class DriveCommentRepliesApi:
    """drive drive.comment_replies API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, comment_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveCommentReplyListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}/replies", query))

    def create(self, node_id: str, comment_id: str, body: CreateCommentReplyRequest) -> DriveCommentReplyHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}/replies", json=body)

    def retrieve(self, node_id: str, comment_id: str, reply_id: str) -> DriveCommentReplyHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}/replies/{serialize_path_parameter(reply_id, {'name': 'replyId', 'style': 'simple', 'explode': False})}")

    def update(self, node_id: str, comment_id: str, reply_id: str, body: UpdateCommentReplyRequest) -> DriveCommentReplyHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}/replies/{serialize_path_parameter(reply_id, {'name': 'replyId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, node_id: str, comment_id: str, reply_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/comments/{serialize_path_parameter(comment_id, {'name': 'commentId', 'style': 'simple', 'explode': False})}/replies/{serialize_path_parameter(reply_id, {'name': 'replyId', 'style': 'simple', 'explode': False})}")

class DriveDownloadGrantsApi:
    """drive drive.download_grants API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, node_id: str, body: Optional[CreateDownloadGrantRequest] = None) -> CreateDownloadUrlHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/download_grants", json=body)

class DriveNodeLabelsApi:
    """drive drive.node_labels API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, label_key: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> NodeLabelListHttpResponse:
        """List labels applied to a node"""
        query = build_query_string([
            {'name': 'labelKey', 'value': label_key, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/labels", query))

    def update(self, node_id: str, label_id: str, body: ApplyNodeLabelRequest) -> NodeLabelHttpResponse:
        """Apply a label to a node"""
        return self._client.put(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/labels/{serialize_path_parameter(label_id, {'name': 'labelId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, node_id: str, label_id: str) -> None:
        """Remove a label from a node"""
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/labels/{serialize_path_parameter(label_id, {'name': 'labelId', 'style': 'simple', 'explode': False})}")

class DrivePermissionsApi:
    """drive drive.permissions API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.effective = DrivePermissionsEffectiveApi(client)


    def list(self, node_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DrivePermissionListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions", query))

    def create(self, node_id: str, body: CreatePermissionRequest) -> DrivePermissionHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions", json=body)

    def delete(self, node_id: str, permission_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions/{serialize_path_parameter(permission_id, {'name': 'permissionId', 'style': 'simple', 'explode': False})}")

    def update(self, node_id: str, permission_id: str, body: UpdatePermissionRequest) -> DrivePermissionHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions/{serialize_path_parameter(permission_id, {'name': 'permissionId', 'style': 'simple', 'explode': False})}", json=body)

    def retrieve(self, node_id: str, permission_id: str) -> DrivePermissionHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions/{serialize_path_parameter(permission_id, {'name': 'permissionId', 'style': 'simple', 'explode': False})}")

class DrivePermissionsEffectiveApi:
    """drive drive.permissions.effective API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> EffectivePermissionListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/permissions/effective", query))

class DriveNodePropertiesApi:
    """drive drive.node_properties API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, visibility: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveNodePropertyListHttpResponse:
        """List node custom properties"""
        query = build_query_string([
            {'name': 'visibility', 'value': visibility, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/properties", query))

    def update(self, node_id: str, property_key: str, body: SetNodePropertyRequest) -> DriveNodePropertyHttpResponse:
        """Create or update a node custom property"""
        return self._client.put(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/properties/{serialize_path_parameter(property_key, {'name': 'propertyKey', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, node_id: str, property_key: str, visibility: Optional[str] = None) -> None:
        """Delete a node custom property"""
        query = build_query_string([
            {'name': 'visibility', 'value': visibility, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.delete(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/properties/{serialize_path_parameter(property_key, {'name': 'propertyKey', 'style': 'simple', 'explode': False})}", query))

class DriveShareLinksApi:
    """drive drive.share_links API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, node_id: str, body: CreateShareLinkRequest) -> CreateShareLinkHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/share_links", json=body)

    def list(self, node_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> ShareLinkListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/share_links", query))

    def claim(self, token: str) -> ClaimShareLinkHttpResponse:
        return self._client.post(f"/app/v3/api/drive/share_links/{serialize_path_parameter(token, {'name': 'token', 'style': 'simple', 'explode': False})}/claim")

    def delete(self, share_link_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/share_links/{serialize_path_parameter(share_link_id, {'name': 'shareLinkId', 'style': 'simple', 'explode': False})}")

    def update(self, share_link_id: str, body: UpdateShareLinkRequest) -> ShareLinkHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/share_links/{serialize_path_parameter(share_link_id, {'name': 'shareLinkId', 'style': 'simple', 'explode': False})}", json=body)

    def retrieve(self, share_link_id: str) -> ShareLinkHttpResponse:
        return self._client.get(f"/app/v3/api/drive/share_links/{serialize_path_parameter(share_link_id, {'name': 'shareLinkId', 'style': 'simple', 'explode': False})}")

class DriveTrashApi:
    """drive drive.trash API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, node_id: str, body: NodeCommandRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/trash", json=body)

    def list(self, space_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None, parent_node_id: Optional[str] = None, sort_by: Optional[str] = None, sort_order: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'parentNodeId', 'value': parent_node_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortBy', 'value': sort_by, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortOrder', 'value': sort_order, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/trash", query))

    def restore(self, node_id: str, body: NodeCommandRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/trash/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/restore", json=body)

    def empty(self, body: EmptyTrashRequest) -> EmptyTrashHttpResponse:
        return self._client.post(f"/app/v3/api/drive/trash/empty", json=body)

class DriveVersionsApi:
    """drive drive.versions API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> FileVersionListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/versions", query))

    def delete(self, node_id: str, version_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/versions/{serialize_path_parameter(version_id, {'name': 'versionId', 'style': 'simple', 'explode': False})}")

    def retrieve(self, node_id: str, version_id: str) -> FileVersionHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/versions/{serialize_path_parameter(version_id, {'name': 'versionId', 'style': 'simple', 'explode': False})}")

    def restore(self, node_id: str, version_id: str, body: NodeCommandRequest) -> DriveNodeHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/versions/{serialize_path_parameter(version_id, {'name': 'versionId', 'style': 'simple', 'explode': False})}/restore", json=body)

class DrivePropertyNodesApi:
    """drive drive.property_nodes API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, property_key: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveNodeListHttpResponse:
        """List nodes carrying an app_public property"""
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/properties/{serialize_path_parameter(property_key, {'name': 'propertyKey', 'style': 'simple', 'explode': False})}/nodes", query))

class DriveRecentApi:
    """drive drive.recent API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, space_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None, sort_by: Optional[str] = None, sort_order: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortBy', 'value': sort_by, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortOrder', 'value': sort_order, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/recent", query))

class DriveSearchApi:
    """drive drive.search API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, q: Optional[str] = None, space_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'q', 'value': q, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/search", query))

class DriveSharedWithMeApi:
    """drive drive.shared_with_me API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, space_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None, sort_by: Optional[str] = None, sort_order: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'spaceId', 'value': space_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortBy', 'value': sort_by, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sortOrder', 'value': sort_order, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/shared_with_me", query))

class DriveSandboxesApi:
    """drive drive.sandboxes API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, page: Optional[int] = None, page_size: Optional[int] = None) -> DriveSandboxVolumeListHttpResponse:
        query = build_query_string([
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/sandboxes", query))

class DriveSandboxEntriesApi:
    """drive drive.sandbox_entries API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, sandbox_id: str, parent_path: Optional[str] = None, cursor: Optional[str] = None, page_size: Optional[int] = None) -> DriveSandboxEntryListHttpResponse:
        query = build_query_string([
            {'name': 'parent_path', 'value': parent_path, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/entries", query))

    def update(self, sandbox_id: str, entry_id: str, body: UpdateDriveSandboxEntryRequest, if_match: str, idempotency_key: str) -> DriveSandboxEntryHttpResponse:
        request_headers = build_request_headers(
            {
                'If-Match': {'value': if_match, 'style': 'simple', 'explode': False},
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.patch(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/entries/{serialize_path_parameter(entry_id, {'name': 'entryId', 'style': 'simple', 'explode': False})}", json=body, headers=request_headers)

    def purge(self, sandbox_id: str, entry_id: str, body: PurgeDriveSandboxEntryRequest, if_match: str, idempotency_key: str) -> DriveSandboxMutationCommandHttpResponse:
        request_headers = build_request_headers(
            {
                'If-Match': {'value': if_match, 'style': 'simple', 'explode': False},
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.post(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/entries/{serialize_path_parameter(entry_id, {'name': 'entryId', 'style': 'simple', 'explode': False})}/purge", json=body, headers=request_headers)

class DriveSandboxDirectoriesApi:
    """drive drive.sandbox_directories API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, sandbox_id: str, body: CreateDriveSandboxDirectoryRequest, idempotency_key: str) -> DriveSandboxEntryHttpResponse:
        request_headers = build_request_headers(
            {
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.post(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/directories", json=body, headers=request_headers)

class DriveSandboxFilesApi:
    """drive drive.sandbox_files API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, sandbox_id: str, body: CreateDriveSandboxFileRequest, idempotency_key: str) -> DriveSandboxEntryHttpResponse:
        request_headers = build_request_headers(
            {
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.post(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/files", json=body, headers=request_headers)

class DriveSandboxFileContentsApi:
    """drive drive.sandbox_file_contents API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, sandbox_id: str, entry_id: str, logical_path: str, encoding: Optional[str] = None) -> DriveSandboxFileContentHttpResponse:
        query = build_query_string([
            {'name': 'logical_path', 'value': logical_path, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'encoding', 'value': encoding, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/files/{serialize_path_parameter(entry_id, {'name': 'entryId', 'style': 'simple', 'explode': False})}/content", query))

    def update(self, sandbox_id: str, entry_id: str, body: UpdateDriveSandboxFileContentRequest, if_match: str, idempotency_key: str) -> DriveSandboxEntryHttpResponse:
        request_headers = build_request_headers(
            {
                'If-Match': {'value': if_match, 'style': 'simple', 'explode': False},
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.put(f"/app/v3/api/drive/sandboxes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/files/{serialize_path_parameter(entry_id, {'name': 'entryId', 'style': 'simple', 'explode': False})}/content", json=body, headers=request_headers)

class DriveSpacesApi:
    """drive drive.spaces API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, owner_subject_type: Optional[str] = None, owner_subject_id: Optional[str] = None, space_type: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveSpaceListHttpResponse:
        query = build_query_string([
            {'name': 'ownerSubjectType', 'value': owner_subject_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'ownerSubjectId', 'value': owner_subject_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'spaceType', 'value': space_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/spaces", query))

    def create(self, body: CreateSpaceRequest) -> DriveSpaceHttpResponse:
        return self._client.post(f"/app/v3/api/drive/spaces", json=body)

    def retrieve(self, space_id: str) -> DriveSpaceHttpResponse:
        return self._client.get(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}")

    def update(self, space_id: str, body: UpdateSpaceRequest) -> DriveSpaceHttpResponse:
        return self._client.patch(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, space_id: str) -> None:
        return self._client.delete(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}")

class DriveWebsiteRootsApi:
    """drive drive.website_roots API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.syncs = DriveWebsiteRootsSyncsApi(client)
        self.generations = DriveWebsiteRootsGenerationsApi(client)


    def list(self, space_id: str, page_size: Optional[int] = None, cursor: Optional[str] = None) -> WebsiteRootListHttpResponse:
        query = build_query_string([
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}/website_roots", query))

    def create(self, space_id: str, body: CreateWebsiteRootRequest) -> WebsiteRootHttpResponse:
        return self._client.post(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}/website_roots", json=body)

    def retrieve(self, root_uuid: str) -> WebsiteRootHttpResponse:
        return self._client.get(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}")

class DriveWebsiteRootsSyncsApi:
    """drive drive.website_roots.syncs API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def create(self, root_uuid: str, body: CreateWebsiteSyncRequest, idempotency_key: str) -> WebsiteSyncHttpResponse:
        """Create an isolated atomic website synchronization"""
        request_headers = build_request_headers(
            {
                'Idempotency-Key': {'value': idempotency_key, 'style': 'simple', 'explode': False},
            },
            {}
        )
        return self._client.post(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}/syncs", json=body, headers=request_headers)

    def retrieve(self, root_uuid: str, sync_id: str) -> WebsiteSyncHttpResponse:
        """Retrieve an atomic website synchronization"""
        return self._client.get(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}/syncs/{serialize_path_parameter(sync_id, {'name': 'syncId', 'style': 'simple', 'explode': False})}")

    def finalize(self, root_uuid: str, sync_id: str, body: WebsiteSyncVersionRequest) -> WebsiteSyncActivationHttpResponse:
        """Validate and atomically activate a complete website tree"""
        return self._client.post(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}/syncs/{serialize_path_parameter(sync_id, {'name': 'syncId', 'style': 'simple', 'explode': False})}/finalize", json=body)

    def abort(self, root_uuid: str, sync_id: str, body: WebsiteSyncVersionRequest) -> WebsiteSyncHttpResponse:
        """Abort an unactivated website synchronization"""
        return self._client.post(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}/syncs/{serialize_path_parameter(sync_id, {'name': 'syncId', 'style': 'simple', 'explode': False})}/abort", json=body)

class DriveWebsiteRootsGenerationsApi:
    """drive drive.website_roots.generations API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def activate(self, root_uuid: str, generation: str, body: ActivateWebsiteGenerationRequest) -> WebsiteGenerationActivationHttpResponse:
        """Activate a retained website generation as a new logical generation"""
        return self._client.post(f"/app/v3/api/drive/website_roots/{serialize_path_parameter(root_uuid, {'name': 'rootUuid', 'style': 'simple', 'explode': False})}/generations/{serialize_path_parameter(generation, {'name': 'generation', 'style': 'simple', 'explode': False})}/activate", json=body)

class DriveMoveDestinationsApi:
    """drive drive.move_destinations API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, space_id: str, exclude_node_ids: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveNodeListHttpResponse:
        query = build_query_string([
            {'name': 'excludeNodeIds', 'value': exclude_node_ids, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/spaces/{serialize_path_parameter(space_id, {'name': 'spaceId', 'style': 'simple', 'explode': False})}/move_destinations", query))

class DriveUploadSessionsApi:
    """drive drive.upload_sessions API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.parts = DriveUploadSessionsPartsApi(client)


    def create(self, body: CreateUploadSessionRequest) -> DriveUploadSessionHttpResponse:
        return self._client.post(f"/app/v3/api/drive/upload_sessions", json=body)

    def retrieve(self, upload_session_id: str) -> DriveUploadSessionHttpResponse:
        return self._client.get(f"/app/v3/api/drive/upload_sessions/{serialize_path_parameter(upload_session_id, {'name': 'uploadSessionId', 'style': 'simple', 'explode': False})}")

    def abort(self, upload_session_id: str, body: NodeCommandRequest) -> DriveUploadSessionHttpResponse:
        return self._client.post(f"/app/v3/api/drive/upload_sessions/{serialize_path_parameter(upload_session_id, {'name': 'uploadSessionId', 'style': 'simple', 'explode': False})}/abort", json=body)

    def complete(self, upload_session_id: str, body: CompleteUploadSessionRequest) -> DriveUploadSessionHttpResponse:
        return self._client.post(f"/app/v3/api/drive/upload_sessions/{serialize_path_parameter(upload_session_id, {'name': 'uploadSessionId', 'style': 'simple', 'explode': False})}/complete", json=body)

class DriveUploadSessionsPartsApi:
    """drive drive.upload_sessions.parts API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def update(self, upload_session_id: str, part_no: int, body: PresignUploadPartRequest) -> PresignedUploadPartHttpResponse:
        return self._client.put(f"/app/v3/api/drive/upload_sessions/{serialize_path_parameter(upload_session_id, {'name': 'uploadSessionId', 'style': 'simple', 'explode': False})}/parts/{serialize_path_parameter(part_no, {'name': 'partNo', 'style': 'simple', 'explode': False})}", json=body)

class DriveWatchChannelsApi:
    """drive drive.watch_channels API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, resource_type: Optional[str] = None, lifecycle_status: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> DriveWatchChannelListHttpResponse:
        """List Drive watch channels"""
        query = build_query_string([
            {'name': 'resourceType', 'value': resource_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'lifecycleStatus', 'value': lifecycle_status, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/drive/watch_channels", query))

    def retrieve(self, channel_id: str) -> DriveWatchChannelHttpResponse:
        """Get a Drive watch channel"""
        return self._client.get(f"/app/v3/api/drive/watch_channels/{serialize_path_parameter(channel_id, {'name': 'channelId', 'style': 'simple', 'explode': False})}")

    def stop(self, channel_id: str, body: StopWatchChannelRequest) -> StopWatchChannelHttpResponse:
        """Stop a Drive watch channel"""
        return self._client.post(f"/app/v3/api/drive/watch_channels/{serialize_path_parameter(channel_id, {'name': 'channelId', 'style': 'simple', 'explode': False})}/stop", json=body)

class DriveDownloadPackagesApi:
    """drive drive.download_packages API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.download_urls = DriveDownloadPackagesDownloadUrlsApi(client)


    def create(self, body: CreateDownloadPackageRequest) -> DownloadPackageHttpResponse:
        return self._client.post(f"/app/v3/api/drive/download_packages", json=body)

class DriveDownloadPackagesDownloadUrlsApi:
    """drive drive.download_packages.download_urls API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self, package_id: str) -> DownloadPackageHttpResponse:
        return self._client.get(f"/app/v3/api/drive/download_packages/{serialize_path_parameter(package_id, {'name': 'packageId', 'style': 'simple', 'explode': False})}/download_url")

class DriveArchiveEntriesApi:
    """drive drive.archive_entries API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, node_id: str) -> ArchiveEntryListHttpResponse:
        return self._client.get(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/archive_entries")

    def extract(self, node_id: str, body: ExtractArchiveEntriesRequest) -> ExtractArchiveEntriesHttpResponse:
        return self._client.post(f"/app/v3/api/drive/nodes/{serialize_path_parameter(node_id, {'name': 'nodeId', 'style': 'simple', 'explode': False})}/archive_entries/extract", json=body)

class DriveUploaderApi:
    """drive drive.uploader API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.uploads = DriveUploaderUploadsApi(client)


class DriveUploaderUploadsApi:
    """drive drive.uploader.uploads API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.parts = DriveUploaderUploadsPartsApi(client)


    def create(self, body: PrepareUploaderUploadRequest) -> PrepareUploaderUploadHttpResponse:
        return self._client.post(f"/app/v3/api/drive/uploader/uploads", json=body)

class DriveUploaderUploadsPartsApi:
    """drive drive.uploader.uploads.parts API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def update(self, upload_item_id: str, part_no: int, body: MarkUploaderPartUploadedRequest) -> UploaderUploadPartHttpResponse:
        return self._client.put(f"/app/v3/api/drive/uploader/uploads/{serialize_path_parameter(upload_item_id, {'name': 'uploadItemId', 'style': 'simple', 'explode': False})}/parts/{serialize_path_parameter(part_no, {'name': 'partNo', 'style': 'simple', 'explode': False})}", json=body)

class DriveAssetsApi:
    """drive drive.assets API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, cursor: Optional[str] = None, page_size: Optional[int] = None, kind: Optional[str] = None, source_type: Optional[str] = None, q: Optional[str] = None) -> AssetListHttpResponse:
        """List global assets"""
        query = build_query_string([
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'kind', 'value': kind, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'sourceType', 'value': source_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'q', 'value': q, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/app/v3/api/assets", query))

    def create(self, body: CreateAssetRequest) -> AssetItemHttpResponse:
        """Create a global asset metadata record"""
        return self._client.post(f"/app/v3/api/assets", json=body)

    def retrieve(self, asset_id: str) -> AssetItemHttpResponse:
        """Get a global asset"""
        return self._client.get(f"/app/v3/api/assets/{serialize_path_parameter(asset_id, {'name': 'assetId', 'style': 'simple', 'explode': False})}")

    def update(self, asset_id: str, body: UpdateAssetRequest) -> AssetItemHttpResponse:
        """Update a global asset"""
        return self._client.patch(f"/app/v3/api/assets/{serialize_path_parameter(asset_id, {'name': 'assetId', 'style': 'simple', 'explode': False})}", json=body)

    def archive(self, asset_id: str, body: AssetActionRequest) -> AssetItemHttpResponse:
        """Archive a global asset"""
        return self._client.post(f"/app/v3/api/assets/{serialize_path_parameter(asset_id, {'name': 'assetId', 'style': 'simple', 'explode': False})}/archive", json=body)

    def restore(self, asset_id: str, body: AssetActionRequest) -> AssetItemHttpResponse:
        """Restore an archived global asset"""
        return self._client.post(f"/app/v3/api/assets/{serialize_path_parameter(asset_id, {'name': 'assetId', 'style': 'simple', 'explode': False})}/restore", json=body)
