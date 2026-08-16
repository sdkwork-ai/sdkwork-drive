package generated

type Operation struct {
	Method string
	Path   string
}

const (
	SdkName         = "sdkwork-drive-backend-sdk"
	PackageName     = "sdkwork-drive-backend-sdk-generated-go"
	StandardProfile = "sdkwork-v3"
	BaseURL         = "http://127.0.0.1:18080"
	ApiPrefix       = "/backend/v3/api"
)

var Operations = map[string]Operation{
	"auditEvents.list": {Method: "GET", Path: "/backend/v3/api/drive/audit_events"},
	"downloadPackages.list": {Method: "GET", Path: "/backend/v3/api/drive/download_packages"},
	"labels.create": {Method: "POST", Path: "/backend/v3/api/drive/labels"},
	"labels.delete": {Method: "DELETE", Path: "/backend/v3/api/drive/labels/{labelId}"},
	"labels.get": {Method: "GET", Path: "/backend/v3/api/drive/labels/{labelId}"},
	"labels.list": {Method: "GET", Path: "/backend/v3/api/drive/labels"},
	"labels.update": {Method: "PATCH", Path: "/backend/v3/api/drive/labels/{labelId}"},
	"maintenance.jobs.list": {Method: "GET", Path: "/backend/v3/api/drive/maintenance/jobs"},
	"maintenance.objectSweep.start": {Method: "POST", Path: "/backend/v3/api/drive/maintenance/object_sweep"},
	"maintenance.uploadSessionSweep.start": {Method: "POST", Path: "/backend/v3/api/drive/maintenance/upload_session_sweep"},
	"quotas.summary": {Method: "GET", Path: "/backend/v3/api/drive/quotas"},
	"spaces.admin.list": {Method: "GET", Path: "/backend/v3/api/drive/spaces"},
	"storageProviderBindings.default.get": {Method: "GET", Path: "/backend/v3/api/drive/storage_provider_bindings/default"},
	"storageProviderBindings.default.set": {Method: "PUT", Path: "/backend/v3/api/drive/storage_provider_bindings/default"},
	"storageProviders.activate": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers/{providerId}/activate"},
	"storageProviders.bucket.create": {Method: "PUT", Path: "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
	"storageProviders.bucket.delete": {Method: "DELETE", Path: "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
	"storageProviders.bucket.head": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers/{providerId}/bucket"},
	"storageProviders.capabilities.get": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers/{providerId}/capabilities"},
	"storageProviders.create": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers"},
	"storageProviders.credentials.rotate": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers/{providerId}/credentials/rotate"},
	"storageProviders.deactivate": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers/{providerId}/deactivate"},
	"storageProviders.delete": {Method: "DELETE", Path: "/backend/v3/api/drive/storage_providers/{providerId}"},
	"storageProviders.get": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers/{providerId}"},
	"storageProviders.list": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers"},
	"storageProviders.objects.copy": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers/{providerId}/objects/copy"},
	"storageProviders.objects.delete": {Method: "DELETE", Path: "/backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}"},
	"storageProviders.objects.head": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}"},
	"storageProviders.objects.list": {Method: "GET", Path: "/backend/v3/api/drive/storage_providers/{providerId}/objects"},
	"storageProviders.test": {Method: "POST", Path: "/backend/v3/api/drive/storage_providers/{providerId}/test"},
	"storageProviders.update": {Method: "PATCH", Path: "/backend/v3/api/drive/storage_providers/{providerId}"},
}
