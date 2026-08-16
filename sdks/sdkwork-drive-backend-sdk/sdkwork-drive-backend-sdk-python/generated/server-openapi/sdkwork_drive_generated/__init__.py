SDK_NAME = "sdkwork-drive-backend-sdk"
PACKAGE_NAME = "sdkwork-drive-backend-sdk-generated-python"
STANDARD_PROFILE = "sdkwork-v3"
BASE_URL = "http://127.0.0.1:18080"
API_PREFIX = "/backend/v3/api"

OPERATIONS = {
    "auditEvents.list": {"method": "GET", "path": "/backend/v3/api/drive/audit_events"},
    "downloadPackages.list": {"method": "GET", "path": "/backend/v3/api/drive/download_packages"},
    "labels.create": {"method": "POST", "path": "/backend/v3/api/drive/labels"},
    "labels.delete": {"method": "DELETE", "path": "/backend/v3/api/drive/labels/{labelId}"},
    "labels.get": {"method": "GET", "path": "/backend/v3/api/drive/labels/{labelId}"},
    "labels.list": {"method": "GET", "path": "/backend/v3/api/drive/labels"},
    "labels.update": {"method": "PATCH", "path": "/backend/v3/api/drive/labels/{labelId}"},
    "maintenance.jobs.list": {"method": "GET", "path": "/backend/v3/api/drive/maintenance/jobs"},
    "maintenance.objectSweep.start": {"method": "POST", "path": "/backend/v3/api/drive/maintenance/object_sweep"},
    "maintenance.uploadSessionSweep.start": {"method": "POST", "path": "/backend/v3/api/drive/maintenance/upload_session_sweep"},
    "quotas.summary": {"method": "GET", "path": "/backend/v3/api/drive/quotas"},
    "spaces.admin.list": {"method": "GET", "path": "/backend/v3/api/drive/spaces"},
    "storageProviderBindings.default.get": {"method": "GET", "path": "/backend/v3/api/drive/storage_provider_bindings/default"},
    "storageProviderBindings.default.set": {"method": "PUT", "path": "/backend/v3/api/drive/storage_provider_bindings/default"},
    "storageProviders.activate": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers/{providerId}/activate"},
    "storageProviders.bucket.create": {"method": "PUT", "path": "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
    "storageProviders.bucket.delete": {"method": "DELETE", "path": "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
    "storageProviders.bucket.head": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
    "storageProviders.capabilities.get": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers/{providerId}/capabilities"},
    "storageProviders.create": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers"},
    "storageProviders.credentials.rotate": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers/{providerId}/credentials/rotate"},
    "storageProviders.deactivate": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers/{providerId}/deactivate"},
    "storageProviders.delete": {"method": "DELETE", "path": "/backend/v3/api/drive/storage_providers/{providerId}"},
    "storageProviders.get": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers/{providerId}"},
    "storageProviders.list": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers"},
    "storageProviders.objects.copy": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers/{providerId}/objects/copy"},
    "storageProviders.objects.delete": {"method": "DELETE", "path": "/backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}"},
    "storageProviders.objects.head": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}"},
    "storageProviders.objects.list": {"method": "GET", "path": "/backend/v3/api/drive/storage_providers/{providerId}/objects"},
    "storageProviders.test": {"method": "POST", "path": "/backend/v3/api/drive/storage_providers/{providerId}/test"},
    "storageProviders.update": {"method": "PATCH", "path": "/backend/v3/api/drive/storage_providers/{providerId}"},
}
