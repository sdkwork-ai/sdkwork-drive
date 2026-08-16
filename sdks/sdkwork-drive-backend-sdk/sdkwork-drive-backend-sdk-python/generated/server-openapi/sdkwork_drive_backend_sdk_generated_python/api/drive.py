from typing import Any, Dict, List, Optional
from ..http_client import HttpClient
from ..models import AuditEventsListResponse, CreateLabelRequest, CreateSandboxGrantRequest, CreateSandboxVolumeRequest, DownloadPackagesListResponse, LabelsCreateResponse201, LabelsListResponse, LabelsRetrieveResponse, LabelsUpdateResponse, MaintenanceAbandonedUploadTaskSweepResponse, MaintenanceExpiredUploadContentSweepResponse, MaintenanceJobsListResponse, MaintenanceObjectSweepResponse, MaintenanceUploadSessionSweepResponse, QuotasRetrieveResponse, QuotasUpdateResponse, SandboxGrantsCreateResponse201, SandboxGrantsListResponse, SandboxGrantsRetrieveResponse, SandboxGrantsUpdateResponse, SandboxVolumesCreateResponse201, SandboxVolumesListResponse, SandboxVolumesRetrieveResponse, SandboxVolumesUpdateResponse, SpacesAdminListResponse, SweepObjectStoreRequest, SweepUploadSessionsRequest, UpdateLabelRequest, UpdateQuotaPolicyRequest, UpdateSandboxGrantRequest, UpdateSandboxVolumeRequest

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



class DriveApi:
    """drive drive API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.audit_events = DriveAuditEventsApi(client)
        self.labels = DriveLabelsApi(client)
        self.maintenance = DriveMaintenanceApi(client)
        self.quotas = DriveQuotasApi(client)
        self.spaces = DriveSpacesApi(client)
        self.download_packages = DriveDownloadPackagesApi(client)
        self.sandbox_volumes = DriveSandboxVolumesApi(client)
        self.sandbox_grants = DriveSandboxGrantsApi(client)


class DriveAuditEventsApi:
    """drive drive.audit_events API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, action: Optional[str] = None, resource_type: Optional[str] = None, resource_id: Optional[str] = None, correlation_id: Optional[str] = None, trace_id: Optional[str] = None, page: Optional[int] = None, page_size: Optional[int] = None) -> AuditEventsListResponse:
        query = build_query_string([
            {'name': 'action', 'value': action, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'resourceType', 'value': resource_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'resourceId', 'value': resource_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'correlationId', 'value': correlation_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'traceId', 'value': trace_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/audit_events", query))

class DriveLabelsApi:
    """drive drive.labels API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, lifecycle_status: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> LabelsListResponse:
        """List Drive label definitions"""
        query = build_query_string([
            {'name': 'lifecycleStatus', 'value': lifecycle_status, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/labels", query))

    def create(self, body: CreateLabelRequest) -> LabelsCreateResponse201:
        """Create a Drive label definition"""
        return self._client.post(f"/backend/v3/api/drive/labels", json=body)

    def retrieve(self, label_id: str) -> LabelsRetrieveResponse:
        """Get a Drive label definition"""
        return self._client.get(f"/backend/v3/api/drive/labels/{serialize_path_parameter(label_id, {'name': 'labelId', 'style': 'simple', 'explode': False})}")

    def update(self, label_id: str, body: UpdateLabelRequest) -> LabelsUpdateResponse:
        """Update a Drive label definition"""
        return self._client.patch(f"/backend/v3/api/drive/labels/{serialize_path_parameter(label_id, {'name': 'labelId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, label_id: str) -> None:
        """Delete a Drive label definition"""
        return self._client.delete(f"/backend/v3/api/drive/labels/{serialize_path_parameter(label_id, {'name': 'labelId', 'style': 'simple', 'explode': False})}")

class DriveMaintenanceApi:
    """drive drive.maintenance API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.jobs = DriveMaintenanceJobsApi(client)


    def object_sweep(self, body: SweepObjectStoreRequest) -> MaintenanceObjectSweepResponse:
        return self._client.post(f"/backend/v3/api/drive/maintenance/object_sweep", json=body)

    def upload_session_sweep(self, body: SweepUploadSessionsRequest) -> MaintenanceUploadSessionSweepResponse:
        return self._client.post(f"/backend/v3/api/drive/maintenance/upload_session_sweep", json=body)

    def expired_upload_content_sweep(self, body: SweepUploadSessionsRequest) -> MaintenanceExpiredUploadContentSweepResponse:
        return self._client.post(f"/backend/v3/api/drive/maintenance/expired_upload_content_sweep", json=body)

    def abandoned_upload_task_sweep(self, body: SweepUploadSessionsRequest) -> MaintenanceAbandonedUploadTaskSweepResponse:
        return self._client.post(f"/backend/v3/api/drive/maintenance/abandoned_upload_task_sweep", json=body)

class DriveMaintenanceJobsApi:
    """drive drive.maintenance.jobs API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, job_type: Optional[str] = None, status: Optional[str] = None, operator_id: Optional[str] = None, page: Optional[int] = None, page_size: Optional[int] = None) -> MaintenanceJobsListResponse:
        query = build_query_string([
            {'name': 'jobType', 'value': job_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'status', 'value': status, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'operatorId', 'value': operator_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/maintenance/jobs", query))

class DriveQuotasApi:
    """drive drive.quotas API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def retrieve(self) -> QuotasRetrieveResponse:
        return self._client.get(f"/backend/v3/api/drive/quotas")

    def update(self, body: UpdateQuotaPolicyRequest) -> QuotasUpdateResponse:
        """Update tenant quota policy"""
        return self._client.put(f"/backend/v3/api/drive/quotas", json=body)

class DriveSpacesApi:
    """drive drive.spaces API client."""

    def __init__(self, client: HttpClient):
        self._client = client
        self.admin = DriveSpacesAdminApi(client)


class DriveSpacesAdminApi:
    """drive drive.spaces.admin API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, owner_subject_type: Optional[str] = None, owner_subject_id: Optional[str] = None, page_size: Optional[int] = None, cursor: Optional[str] = None) -> SpacesAdminListResponse:
        query = build_query_string([
            {'name': 'ownerSubjectType', 'value': owner_subject_type, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'ownerSubjectId', 'value': owner_subject_id, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'cursor', 'value': cursor, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/spaces", query))

class DriveDownloadPackagesApi:
    """drive drive.download_packages API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, state: Optional[str] = None, page: Optional[int] = None, page_size: Optional[int] = None) -> DownloadPackagesListResponse:
        query = build_query_string([
            {'name': 'state', 'value': state, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/download_packages", query))

class DriveSandboxVolumesApi:
    """drive drive.sandbox_volumes API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, lifecycle_status: Optional[str] = None, provider_kind: Optional[str] = None, page: Optional[int] = None, page_size: Optional[int] = None) -> SandboxVolumesListResponse:
        """List server sandbox volumes"""
        query = build_query_string([
            {'name': 'lifecycle_status', 'value': lifecycle_status, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'provider_kind', 'value': provider_kind, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/sandbox_volumes", query))

    def create(self, body: CreateSandboxVolumeRequest) -> SandboxVolumesCreateResponse201:
        """Create a server sandbox volume"""
        return self._client.post(f"/backend/v3/api/drive/sandbox_volumes", json=body)

    def retrieve(self, sandbox_id: str) -> SandboxVolumesRetrieveResponse:
        """Retrieve a server sandbox volume"""
        return self._client.get(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}")

    def update(self, sandbox_id: str, body: UpdateSandboxVolumeRequest) -> SandboxVolumesUpdateResponse:
        """Update a server sandbox volume"""
        return self._client.patch(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, sandbox_id: str) -> None:
        """Delete a server sandbox volume"""
        return self._client.delete(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}")

class DriveSandboxGrantsApi:
    """drive drive.sandbox_grants API client."""

    def __init__(self, client: HttpClient):
        self._client = client


    def list(self, sandbox_id: str, page: Optional[int] = None, page_size: Optional[int] = None) -> SandboxGrantsListResponse:
        """List explicit sandbox grants"""
        query = build_query_string([
            {'name': 'page', 'value': page, 'style': 'form', 'explode': True, 'allow_reserved': False},
            {'name': 'page_size', 'value': page_size, 'style': 'form', 'explode': True, 'allow_reserved': False},
        ])
        return self._client.get(_append_query_string(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/grants", query))

    def create(self, sandbox_id: str, body: CreateSandboxGrantRequest) -> SandboxGrantsCreateResponse201:
        """Create an explicit sandbox grant"""
        return self._client.post(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/grants", json=body)

    def retrieve(self, sandbox_id: str, grant_id: str) -> SandboxGrantsRetrieveResponse:
        """Retrieve a sandbox grant"""
        return self._client.get(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/grants/{serialize_path_parameter(grant_id, {'name': 'grantId', 'style': 'simple', 'explode': False})}")

    def update(self, sandbox_id: str, grant_id: str, body: UpdateSandboxGrantRequest) -> SandboxGrantsUpdateResponse:
        """Update a sandbox grant"""
        return self._client.patch(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/grants/{serialize_path_parameter(grant_id, {'name': 'grantId', 'style': 'simple', 'explode': False})}", json=body)

    def delete(self, sandbox_id: str, grant_id: str) -> None:
        """Delete a sandbox grant"""
        return self._client.delete(f"/backend/v3/api/drive/sandbox_volumes/{serialize_path_parameter(sandbox_id, {'name': 'sandboxId', 'style': 'simple', 'explode': False})}/grants/{serialize_path_parameter(grant_id, {'name': 'grantId', 'style': 'simple', 'explode': False})}")
