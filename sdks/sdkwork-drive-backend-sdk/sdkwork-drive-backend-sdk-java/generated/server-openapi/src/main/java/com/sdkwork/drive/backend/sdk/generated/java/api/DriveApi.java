package com.sdkwork.drive.backend.sdk.generated.java.api;

import com.fasterxml.jackson.core.type.TypeReference;
import com.sdkwork.drive.backend.sdk.generated.java.http.HttpClient;
import com.sdkwork.drive.backend.sdk.generated.java.model.*;
import java.util.List;
import java.util.Map;

public class DriveApi {
    private final HttpClient client;

    public DriveApi(HttpClient client) {
        this.client = client;
    }

    public AuditEventsListResponse auditEventsList(String action, String resourceType, String resourceId, String correlationId, String traceId, Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("action", action, "form", true, false, null),
            new QueryParameterSpec("resourceType", resourceType, "form", true, false, null),
            new QueryParameterSpec("resourceId", resourceId, "form", true, false, null),
            new QueryParameterSpec("correlationId", correlationId, "form", true, false, null),
            new QueryParameterSpec("traceId", traceId, "form", true, false, null),
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/audit_events"), query));
        return client.convertValue(raw, new TypeReference<AuditEventsListResponse>() {});
    }

    /** List Drive label definitions */
    public LabelsListResponse labelsList(String lifecycleStatus, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("lifecycleStatus", lifecycleStatus, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/labels"), query));
        return client.convertValue(raw, new TypeReference<LabelsListResponse>() {});
    }

    /** Create a Drive label definition */
    public LabelsCreateResponse201 labelsCreate(CreateLabelRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/labels"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<LabelsCreateResponse201>() {});
    }

    /** Get a Drive label definition */
    public LabelsRetrieveResponse labelsRetrieve(String labelId) throws Exception {
        Object raw = client.get(ApiPaths.backendPath("/drive/labels/" + serializePathParameter(labelId, new PathParameterSpec("labelId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<LabelsRetrieveResponse>() {});
    }

    /** Update a Drive label definition */
    public LabelsUpdateResponse labelsUpdate(String labelId, UpdateLabelRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.backendPath("/drive/labels/" + serializePathParameter(labelId, new PathParameterSpec("labelId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<LabelsUpdateResponse>() {});
    }

    /** Delete a Drive label definition */
    public Void labelsDelete(String labelId) throws Exception {
        client.delete(ApiPaths.backendPath("/drive/labels/" + serializePathParameter(labelId, new PathParameterSpec("labelId", "simple", false)) + ""));
        return null;
    }

    public MaintenanceJobsListResponse maintenanceJobsList(String jobType, String status, String operatorId, Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("jobType", jobType, "form", true, false, null),
            new QueryParameterSpec("status", status, "form", true, false, null),
            new QueryParameterSpec("operatorId", operatorId, "form", true, false, null),
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/maintenance/jobs"), query));
        return client.convertValue(raw, new TypeReference<MaintenanceJobsListResponse>() {});
    }

    public MaintenanceObjectSweepResponse maintenanceObjectSweep(SweepObjectStoreRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/maintenance/object_sweep"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<MaintenanceObjectSweepResponse>() {});
    }

    public MaintenanceUploadSessionSweepResponse maintenanceUploadSessionSweep(SweepUploadSessionsRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/maintenance/upload_session_sweep"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<MaintenanceUploadSessionSweepResponse>() {});
    }

    public MaintenanceExpiredUploadContentSweepResponse maintenanceExpiredUploadContentSweep(SweepUploadSessionsRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/maintenance/expired_upload_content_sweep"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<MaintenanceExpiredUploadContentSweepResponse>() {});
    }

    public MaintenanceAbandonedUploadTaskSweepResponse maintenanceAbandonedUploadTaskSweep(SweepUploadSessionsRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/maintenance/abandoned_upload_task_sweep"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<MaintenanceAbandonedUploadTaskSweepResponse>() {});
    }

    public QuotasRetrieveResponse quotasRetrieve() throws Exception {
        Object raw = client.get(ApiPaths.backendPath("/drive/quotas"));
        return client.convertValue(raw, new TypeReference<QuotasRetrieveResponse>() {});
    }

    /** Update tenant quota policy */
    public QuotasUpdateResponse quotasUpdate(UpdateQuotaPolicyRequest body) throws Exception {
        Object raw = client.put(ApiPaths.backendPath("/drive/quotas"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<QuotasUpdateResponse>() {});
    }

    public SpacesAdminListResponse spacesAdminList(String ownerSubjectType, String ownerSubjectId, Integer pageSize, String cursor) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("ownerSubjectType", ownerSubjectType, "form", true, false, null),
            new QueryParameterSpec("ownerSubjectId", ownerSubjectId, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null),
            new QueryParameterSpec("cursor", cursor, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/spaces"), query));
        return client.convertValue(raw, new TypeReference<SpacesAdminListResponse>() {});
    }

    public DownloadPackagesListResponse downloadPackagesList(String state, Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("state", state, "form", true, false, null),
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/download_packages"), query));
        return client.convertValue(raw, new TypeReference<DownloadPackagesListResponse>() {});
    }

    /** List server sandbox volumes */
    public SandboxVolumesListResponse sandboxVolumesList(String lifecycleStatus, String providerKind, Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("lifecycle_status", lifecycleStatus, "form", true, false, null),
            new QueryParameterSpec("provider_kind", providerKind, "form", true, false, null),
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/sandbox_volumes"), query));
        return client.convertValue(raw, new TypeReference<SandboxVolumesListResponse>() {});
    }

    /** Create a server sandbox volume */
    public SandboxVolumesCreateResponse201 sandboxVolumesCreate(CreateSandboxVolumeRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/sandbox_volumes"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<SandboxVolumesCreateResponse201>() {});
    }

    /** Retrieve a server sandbox volume */
    public SandboxVolumesRetrieveResponse sandboxVolumesRetrieve(String sandboxId) throws Exception {
        Object raw = client.get(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<SandboxVolumesRetrieveResponse>() {});
    }

    /** Update a server sandbox volume */
    public SandboxVolumesUpdateResponse sandboxVolumesUpdate(String sandboxId, UpdateSandboxVolumeRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<SandboxVolumesUpdateResponse>() {});
    }

    /** Delete a server sandbox volume */
    public Void sandboxVolumesDelete(String sandboxId) throws Exception {
        client.delete(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + ""));
        return null;
    }

    /** List explicit sandbox grants */
    public SandboxGrantsListResponse sandboxGrantsList(String sandboxId, Integer page, Integer pageSize) throws Exception {
        String query = buildQueryString(List.of(
            new QueryParameterSpec("page", page, "form", true, false, null),
            new QueryParameterSpec("page_size", pageSize, "form", true, false, null)
        ));
        Object raw = client.get(ApiPaths.appendQueryString(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/grants"), query));
        return client.convertValue(raw, new TypeReference<SandboxGrantsListResponse>() {});
    }

    /** Create an explicit sandbox grant */
    public SandboxGrantsCreateResponse201 sandboxGrantsCreate(String sandboxId, CreateSandboxGrantRequest body) throws Exception {
        Object raw = client.post(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/grants"), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<SandboxGrantsCreateResponse201>() {});
    }

    /** Retrieve a sandbox grant */
    public SandboxGrantsRetrieveResponse sandboxGrantsRetrieve(String sandboxId, String grantId) throws Exception {
        Object raw = client.get(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/grants/" + serializePathParameter(grantId, new PathParameterSpec("grantId", "simple", false)) + ""));
        return client.convertValue(raw, new TypeReference<SandboxGrantsRetrieveResponse>() {});
    }

    /** Update a sandbox grant */
    public SandboxGrantsUpdateResponse sandboxGrantsUpdate(String sandboxId, String grantId, UpdateSandboxGrantRequest body) throws Exception {
        Object raw = client.patch(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/grants/" + serializePathParameter(grantId, new PathParameterSpec("grantId", "simple", false)) + ""), body, null, null, "application/json");
        return client.convertValue(raw, new TypeReference<SandboxGrantsUpdateResponse>() {});
    }

    /** Delete a sandbox grant */
    public Void sandboxGrantsDelete(String sandboxId, String grantId) throws Exception {
        client.delete(ApiPaths.backendPath("/drive/sandbox_volumes/" + serializePathParameter(sandboxId, new PathParameterSpec("sandboxId", "simple", false)) + "/grants/" + serializePathParameter(grantId, new PathParameterSpec("grantId", "simple", false)) + ""));
        return null;
    }

    private record PathParameterSpec(String name, String style, boolean explode) {}

    private static String serializePathParameter(Object value, PathParameterSpec spec) {
        if (value == null) {
            return "";
        }
        String style = spec.style() == null || spec.style().isBlank() ? "simple" : spec.style();
        if (value instanceof Iterable<?> iterable) {
            return serializePathArray(spec.name(), iterable, style, spec.explode());
        }
        if (value instanceof Map<?, ?> map) {
            return serializePathObject(spec.name(), map, style, spec.explode());
        }
        return pathPrimitivePrefix(spec.name(), style) + pathEncode(String.valueOf(value));
    }

    private static String serializePathArray(String name, Iterable<?> values, String style, boolean explode) {
        List<String> serialized = new java.util.ArrayList<>();
        for (Object item : values) {
            if (item != null) {
                serialized.add(pathEncode(String.valueOf(item)));
            }
        }
        if (serialized.isEmpty()) {
            return pathPrefix(name, style);
        }
        if ("matrix".equals(style)) {
            if (explode) {
                List<String> parts = new java.util.ArrayList<>();
                for (String item : serialized) {
                    parts.add(";" + name + "=" + item);
                }
                return String.join("", parts);
            }
            return ";" + name + "=" + String.join(",", serialized);
        }
        String separator = explode ? "." : ",";
        return pathPrefix(name, style) + String.join(separator, serialized);
    }

    private static String serializePathObject(String name, Map<?, ?> values, String style, boolean explode) {
        List<String> entries = new java.util.ArrayList<>();
        List<String> exploded = new java.util.ArrayList<>();
        values.forEach((key, value) -> {
            if (value == null) {
                return;
            }
            String escapedKey = pathEncode(String.valueOf(key));
            String escapedValue = pathEncode(String.valueOf(value));
            if (explode) {
                if ("matrix".equals(style)) {
                    exploded.add(";" + escapedKey + "=" + escapedValue);
                } else {
                    exploded.add(escapedKey + "=" + escapedValue);
                }
            } else {
                entries.add(escapedKey);
                entries.add(escapedValue);
            }
        });
        if ("matrix".equals(style)) {
            if (explode) {
                return String.join("", exploded);
            }
            return ";" + name + "=" + String.join(",", entries);
        }
        if (explode) {
            String separator = "label".equals(style) ? "." : ",";
            return pathPrefix(name, style) + String.join(separator, exploded);
        }
        return pathPrefix(name, style) + String.join(",", entries);
    }

    private static String pathPrefix(String name, String style) {
        if ("label".equals(style)) {
            return ".";
        }
        if ("matrix".equals(style)) {
            return ";" + name;
        }
        return "";
    }

    private static String pathPrimitivePrefix(String name, String style) {
        if ("matrix".equals(style)) {
            return ";" + name + "=";
        }
        return pathPrefix(name, style);
    }

    private static String pathEncode(String value) {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8).replace("+", "%20");
    }

    private record QueryParameterSpec(String name, Object value, String style, boolean explode, boolean allowReserved, String contentType) {}

    private static String buildQueryString(List<QueryParameterSpec> parameters) throws Exception {
        List<String> pairs = new java.util.ArrayList<>();
        for (QueryParameterSpec parameter : parameters) {
            appendSerializedParameter(pairs, parameter);
        }
        return String.join("&", pairs);
    }

    private static void appendSerializedParameter(List<String> pairs, QueryParameterSpec parameter) throws Exception {
        if (parameter.value() == null) {
            return;
        }
        if (parameter.contentType() != null && !parameter.contentType().isBlank()) {
            String json = clientObjectMapper().writeValueAsString(parameter.value());
            pairs.add(urlEncode(parameter.name()) + "=" + encodeQueryValue(json, parameter.allowReserved()));
            return;
        }

        String style = parameter.style() == null || parameter.style().isBlank() ? "form" : parameter.style();
        Object value = parameter.value();
        if ("deepObject".equals(style) && value instanceof Map<?, ?> map) {
            appendDeepObjectParameter(pairs, parameter.name(), map, parameter.allowReserved());
        } else if (value instanceof Iterable<?> iterable) {
            appendArrayParameter(pairs, parameter.name(), iterable, style, parameter.explode(), parameter.allowReserved());
        } else if (value instanceof Map<?, ?> map) {
            appendObjectParameter(pairs, parameter.name(), map, style, parameter.explode(), parameter.allowReserved());
        } else {
            pairs.add(urlEncode(parameter.name()) + "=" + encodeQueryValue(String.valueOf(value), parameter.allowReserved()));
        }
    }

    private static void appendArrayParameter(List<String> pairs, String name, Iterable<?> values, String style, boolean explode, boolean allowReserved) {
        List<String> serialized = new java.util.ArrayList<>();
        for (Object item : values) {
            if (item != null) {
                serialized.add(String.valueOf(item));
            }
        }
        if (serialized.isEmpty()) {
            return;
        }
        if ("form".equals(style) && explode) {
            for (String item : serialized) {
                pairs.add(urlEncode(name) + "=" + encodeQueryValue(item, allowReserved));
            }
            return;
        }
        pairs.add(urlEncode(name) + "=" + encodeQueryValue(String.join(",", serialized), allowReserved));
    }

    private static void appendObjectParameter(List<String> pairs, String name, Map<?, ?> values, String style, boolean explode, boolean allowReserved) {
        List<String> serialized = new java.util.ArrayList<>();
        values.forEach((key, value) -> {
            if (value == null) {
                return;
            }
            if ("form".equals(style) && explode) {
                pairs.add(urlEncode(String.valueOf(key)) + "=" + encodeQueryValue(String.valueOf(value), allowReserved));
            } else {
                serialized.add(String.valueOf(key));
                serialized.add(String.valueOf(value));
            }
        });
        if (!serialized.isEmpty()) {
            pairs.add(urlEncode(name) + "=" + encodeQueryValue(String.join(",", serialized), allowReserved));
        }
    }

    private static void appendDeepObjectParameter(List<String> pairs, String name, Map<?, ?> values, boolean allowReserved) {
        values.forEach((key, value) -> {
            if (value != null) {
                pairs.add(urlEncode(name + "[" + key + "]") + "=" + encodeQueryValue(String.valueOf(value), allowReserved));
            }
        });
    }

    private static String encodeQueryValue(String value, boolean allowReserved) {
        String encoded = urlEncode(value);
        if (!allowReserved) {
            return encoded;
        }
        return encoded
            .replace("%3A", ":").replace("%2F", "/").replace("%3F", "?").replace("%23", "#")
            .replace("%5B", "[").replace("%5D", "]").replace("%40", "@").replace("%21", "!")
            .replace("%24", "$").replace("%26", "&").replace("%27", "'").replace("%28", "(")
            .replace("%29", ")").replace("%2A", "*").replace("%2B", "+").replace("%2C", ",")
            .replace("%3B", ";").replace("%3D", "=");
    }

    private static com.fasterxml.jackson.databind.ObjectMapper clientObjectMapper() {
        return new com.fasterxml.jackson.databind.ObjectMapper();
    }


    private static String urlEncode(String value) {
        return java.net.URLEncoder.encode(value, java.nio.charset.StandardCharsets.UTF_8);
    }
}
