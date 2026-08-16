package com.sdkwork.generated;

import java.util.LinkedHashMap;
import java.util.Map;

public final class SdkMetadata {
  public static final String SDK_NAME = "sdkwork-drive-backend-sdk";
  public static final String PACKAGE_NAME = "sdkwork-drive-backend-sdk-generated-java";
  public static final String STANDARD_PROFILE = "sdkwork-v3";
  public static final String BASE_URL = "http://127.0.0.1:18080";
  public static final String API_PREFIX = "/backend/v3/api";

  public static Map<String, String> operations() {
    Map<String, String> operations = new LinkedHashMap<>();
    operations.put("auditEvents.list", "GET /backend/v3/api/drive/audit_events");
    operations.put("downloadPackages.list", "GET /backend/v3/api/drive/download_packages");
    operations.put("labels.create", "POST /backend/v3/api/drive/labels");
    operations.put("labels.delete", "DELETE /backend/v3/api/drive/labels/{labelId}");
    operations.put("labels.get", "GET /backend/v3/api/drive/labels/{labelId}");
    operations.put("labels.list", "GET /backend/v3/api/drive/labels");
    operations.put("labels.update", "PATCH /backend/v3/api/drive/labels/{labelId}");
    operations.put("maintenance.jobs.list", "GET /backend/v3/api/drive/maintenance/jobs");
    operations.put("maintenance.objectSweep.start", "POST /backend/v3/api/drive/maintenance/object_sweep");
    operations.put("maintenance.uploadSessionSweep.start", "POST /backend/v3/api/drive/maintenance/upload_session_sweep");
    operations.put("quotas.summary", "GET /backend/v3/api/drive/quotas");
    operations.put("spaces.admin.list", "GET /backend/v3/api/drive/spaces");
    operations.put("storageProviderBindings.default.get", "GET /backend/v3/api/drive/storage_provider_bindings/default");
    operations.put("storageProviderBindings.default.set", "PUT /backend/v3/api/drive/storage_provider_bindings/default");
    operations.put("storageProviders.activate", "POST /backend/v3/api/drive/storage_providers/{providerId}/activate");
    operations.put("storageProviders.bucket.create", "PUT /backend/v3/api/drive/storage_providers/{providerId}/bucket");
    operations.put("storageProviders.bucket.delete", "DELETE /backend/v3/api/drive/storage_providers/{providerId}/bucket");
    operations.put("storageProviders.bucket.head", "GET /backend/v3/api/drive/storage_providers/{providerId}/bucket");
    operations.put("storageProviders.capabilities.get", "GET /backend/v3/api/drive/storage_providers/{providerId}/capabilities");
    operations.put("storageProviders.create", "POST /backend/v3/api/drive/storage_providers");
    operations.put("storageProviders.credentials.rotate", "POST /backend/v3/api/drive/storage_providers/{providerId}/credentials/rotate");
    operations.put("storageProviders.deactivate", "POST /backend/v3/api/drive/storage_providers/{providerId}/deactivate");
    operations.put("storageProviders.delete", "DELETE /backend/v3/api/drive/storage_providers/{providerId}");
    operations.put("storageProviders.get", "GET /backend/v3/api/drive/storage_providers/{providerId}");
    operations.put("storageProviders.list", "GET /backend/v3/api/drive/storage_providers");
    operations.put("storageProviders.objects.copy", "POST /backend/v3/api/drive/storage_providers/{providerId}/objects/copy");
    operations.put("storageProviders.objects.delete", "DELETE /backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}");
    operations.put("storageProviders.objects.head", "GET /backend/v3/api/drive/storage_providers/{providerId}/objects/{objectKey}");
    operations.put("storageProviders.objects.list", "GET /backend/v3/api/drive/storage_providers/{providerId}/objects");
    operations.put("storageProviders.test", "POST /backend/v3/api/drive/storage_providers/{providerId}/test");
    operations.put("storageProviders.update", "PATCH /backend/v3/api/drive/storage_providers/{providerId}");
    return operations;
  }

  private SdkMetadata() {}
}
