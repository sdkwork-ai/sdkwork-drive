package api

import (
    "encoding/json"
    "fmt"
    "net/url"
    "strings"
    sdktypes "sdkwork-drive-backend-sdk-generated-go/types"
    sdkhttp "sdkwork-drive-backend-sdk-generated-go/http"
)

type DriveApi struct {
    client *sdkhttp.Client
}

func NewDriveApi(client *sdkhttp.Client) *DriveApi {
    return &DriveApi{client: client}
}

func (a *DriveApi) AuditEventsList(action *string, resourceType *string, resourceId *string, correlationId *string, traceId *string, page *int, pageSize *int) (sdktypes.AuditEventsListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "action", Value: func() interface{} { if action == nil { return nil }; return *action }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "resourceType", Value: func() interface{} { if resourceType == nil { return nil }; return *resourceType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "resourceId", Value: func() interface{} { if resourceId == nil { return nil }; return *resourceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "correlationId", Value: func() interface{} { if correlationId == nil { return nil }; return *correlationId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "traceId", Value: func() interface{} { if traceId == nil { return nil }; return *traceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/audit_events"), query), nil, nil)
    if err != nil {
        var zero sdktypes.AuditEventsListResponse
        return zero, err
    }
    return decodeResult[sdktypes.AuditEventsListResponse](raw)
}

// List Drive label definitions
func (a *DriveApi) LabelsList(lifecycleStatus *string, pageSize *int, cursor *string) (sdktypes.LabelsListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "lifecycleStatus", Value: func() interface{} { if lifecycleStatus == nil { return nil }; return *lifecycleStatus }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/labels"), query), nil, nil)
    if err != nil {
        var zero sdktypes.LabelsListResponse
        return zero, err
    }
    return decodeResult[sdktypes.LabelsListResponse](raw)
}

// Create a Drive label definition
func (a *DriveApi) LabelsCreate(body sdktypes.CreateLabelRequest) (sdktypes.LabelsCreateResponse201, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/labels"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.LabelsCreateResponse201
        return zero, err
    }
    return decodeResult[sdktypes.LabelsCreateResponse201](raw)
}

// Get a Drive label definition
func (a *DriveApi) LabelsRetrieve(labelId string) (sdktypes.LabelsRetrieveResponse, error) {
    raw, err := a.client.Get(BackendApiPath(fmt.Sprintf("/drive/labels/%s", SerializePathParameter(labelId, PathParameterSpec{Name: "labelId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.LabelsRetrieveResponse
        return zero, err
    }
    return decodeResult[sdktypes.LabelsRetrieveResponse](raw)
}

// Update a Drive label definition
func (a *DriveApi) LabelsUpdate(labelId string, body sdktypes.UpdateLabelRequest) (sdktypes.LabelsUpdateResponse, error) {
    raw, err := a.client.Patch(BackendApiPath(fmt.Sprintf("/drive/labels/%s", SerializePathParameter(labelId, PathParameterSpec{Name: "labelId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.LabelsUpdateResponse
        return zero, err
    }
    return decodeResult[sdktypes.LabelsUpdateResponse](raw)
}

// Delete a Drive label definition
func (a *DriveApi) LabelsDelete(labelId string) (struct{}, error) {
    raw, err := a.client.Delete(BackendApiPath(fmt.Sprintf("/drive/labels/%s", SerializePathParameter(labelId, PathParameterSpec{Name: "labelId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) MaintenanceJobsList(jobType *string, status *string, operatorId *string, page *int, pageSize *int) (sdktypes.MaintenanceJobsListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "jobType", Value: func() interface{} { if jobType == nil { return nil }; return *jobType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "status", Value: func() interface{} { if status == nil { return nil }; return *status }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "operatorId", Value: func() interface{} { if operatorId == nil { return nil }; return *operatorId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/maintenance/jobs"), query), nil, nil)
    if err != nil {
        var zero sdktypes.MaintenanceJobsListResponse
        return zero, err
    }
    return decodeResult[sdktypes.MaintenanceJobsListResponse](raw)
}

func (a *DriveApi) MaintenanceObjectSweep(body sdktypes.SweepObjectStoreRequest) (sdktypes.MaintenanceObjectSweepResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/maintenance/object_sweep"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.MaintenanceObjectSweepResponse
        return zero, err
    }
    return decodeResult[sdktypes.MaintenanceObjectSweepResponse](raw)
}

func (a *DriveApi) MaintenanceUploadSessionSweep(body sdktypes.SweepUploadSessionsRequest) (sdktypes.MaintenanceUploadSessionSweepResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/maintenance/upload_session_sweep"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.MaintenanceUploadSessionSweepResponse
        return zero, err
    }
    return decodeResult[sdktypes.MaintenanceUploadSessionSweepResponse](raw)
}

func (a *DriveApi) MaintenanceExpiredUploadContentSweep(body sdktypes.SweepUploadSessionsRequest) (sdktypes.MaintenanceExpiredUploadContentSweepResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/maintenance/expired_upload_content_sweep"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.MaintenanceExpiredUploadContentSweepResponse
        return zero, err
    }
    return decodeResult[sdktypes.MaintenanceExpiredUploadContentSweepResponse](raw)
}

func (a *DriveApi) MaintenanceAbandonedUploadTaskSweep(body sdktypes.SweepUploadSessionsRequest) (sdktypes.MaintenanceAbandonedUploadTaskSweepResponse, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/maintenance/abandoned_upload_task_sweep"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.MaintenanceAbandonedUploadTaskSweepResponse
        return zero, err
    }
    return decodeResult[sdktypes.MaintenanceAbandonedUploadTaskSweepResponse](raw)
}

func (a *DriveApi) QuotasRetrieve() (sdktypes.QuotasRetrieveResponse, error) {
    raw, err := a.client.Get(BackendApiPath("/drive/quotas"), nil, nil)
    if err != nil {
        var zero sdktypes.QuotasRetrieveResponse
        return zero, err
    }
    return decodeResult[sdktypes.QuotasRetrieveResponse](raw)
}

// Update tenant quota policy
func (a *DriveApi) QuotasUpdate(body sdktypes.UpdateQuotaPolicyRequest) (sdktypes.QuotasUpdateResponse, error) {
    raw, err := a.client.Put(BackendApiPath("/drive/quotas"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.QuotasUpdateResponse
        return zero, err
    }
    return decodeResult[sdktypes.QuotasUpdateResponse](raw)
}

func (a *DriveApi) SpacesAdminList(ownerSubjectType *string, ownerSubjectId *string, pageSize *int, cursor *string) (sdktypes.SpacesAdminListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "ownerSubjectType", Value: func() interface{} { if ownerSubjectType == nil { return nil }; return *ownerSubjectType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "ownerSubjectId", Value: func() interface{} { if ownerSubjectId == nil { return nil }; return *ownerSubjectId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/spaces"), query), nil, nil)
    if err != nil {
        var zero sdktypes.SpacesAdminListResponse
        return zero, err
    }
    return decodeResult[sdktypes.SpacesAdminListResponse](raw)
}

func (a *DriveApi) DownloadPackagesList(state *string, page *int, pageSize *int) (sdktypes.DownloadPackagesListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "state", Value: func() interface{} { if state == nil { return nil }; return *state }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/download_packages"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DownloadPackagesListResponse
        return zero, err
    }
    return decodeResult[sdktypes.DownloadPackagesListResponse](raw)
}

// List server sandbox volumes
func (a *DriveApi) SandboxVolumesList(lifecycleStatus *sdktypes.SandboxLifecycleStatus, providerKind *sdktypes.SandboxProviderKind, page *int, pageSize *int) (sdktypes.SandboxVolumesListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "lifecycle_status", Value: func() interface{} { if lifecycleStatus == nil { return nil }; return *lifecycleStatus }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "provider_kind", Value: func() interface{} { if providerKind == nil { return nil }; return *providerKind }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath("/drive/sandbox_volumes"), query), nil, nil)
    if err != nil {
        var zero sdktypes.SandboxVolumesListResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxVolumesListResponse](raw)
}

// Create a server sandbox volume
func (a *DriveApi) SandboxVolumesCreate(body sdktypes.CreateSandboxVolumeRequest) (sdktypes.SandboxVolumesCreateResponse201, error) {
    raw, err := a.client.Post(BackendApiPath("/drive/sandbox_volumes"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.SandboxVolumesCreateResponse201
        return zero, err
    }
    return decodeResult[sdktypes.SandboxVolumesCreateResponse201](raw)
}

// Retrieve a server sandbox volume
func (a *DriveApi) SandboxVolumesRetrieve(sandboxId string) (sdktypes.SandboxVolumesRetrieveResponse, error) {
    raw, err := a.client.Get(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.SandboxVolumesRetrieveResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxVolumesRetrieveResponse](raw)
}

// Update a server sandbox volume
func (a *DriveApi) SandboxVolumesUpdate(sandboxId string, body sdktypes.UpdateSandboxVolumeRequest) (sdktypes.SandboxVolumesUpdateResponse, error) {
    raw, err := a.client.Patch(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.SandboxVolumesUpdateResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxVolumesUpdateResponse](raw)
}

// Delete a server sandbox volume
func (a *DriveApi) SandboxVolumesDelete(sandboxId string) (struct{}, error) {
    raw, err := a.client.Delete(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

// List explicit sandbox grants
func (a *DriveApi) SandboxGrantsList(sandboxId string, page *int, pageSize *int) (sdktypes.SandboxGrantsListResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s/grants", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.SandboxGrantsListResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxGrantsListResponse](raw)
}

// Create an explicit sandbox grant
func (a *DriveApi) SandboxGrantsCreate(sandboxId string, body sdktypes.CreateSandboxGrantRequest) (sdktypes.SandboxGrantsCreateResponse201, error) {
    raw, err := a.client.Post(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s/grants", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.SandboxGrantsCreateResponse201
        return zero, err
    }
    return decodeResult[sdktypes.SandboxGrantsCreateResponse201](raw)
}

// Retrieve a sandbox grant
func (a *DriveApi) SandboxGrantsRetrieve(sandboxId string, grantId string) (sdktypes.SandboxGrantsRetrieveResponse, error) {
    raw, err := a.client.Get(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s/grants/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(grantId, PathParameterSpec{Name: "grantId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.SandboxGrantsRetrieveResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxGrantsRetrieveResponse](raw)
}

// Update a sandbox grant
func (a *DriveApi) SandboxGrantsUpdate(sandboxId string, grantId string, body sdktypes.UpdateSandboxGrantRequest) (sdktypes.SandboxGrantsUpdateResponse, error) {
    raw, err := a.client.Patch(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s/grants/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(grantId, PathParameterSpec{Name: "grantId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.SandboxGrantsUpdateResponse
        return zero, err
    }
    return decodeResult[sdktypes.SandboxGrantsUpdateResponse](raw)
}

// Delete a sandbox grant
func (a *DriveApi) SandboxGrantsDelete(sandboxId string, grantId string) (struct{}, error) {
    raw, err := a.client.Delete(BackendApiPath(fmt.Sprintf("/drive/sandbox_volumes/%s/grants/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(grantId, PathParameterSpec{Name: "grantId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

type PathParameterSpec struct {
    Name    string
    Style   string
    Explode bool
}

func SerializePathParameter(value interface{}, spec PathParameterSpec) string {
    if value == nil {
        return ""
    }
    style := spec.Style
    if style == "" {
        style = "simple"
    }

    switch typed := value.(type) {
    case []string:
        return SerializePathArray(spec.Name, stringSliceToInterface(typed), style, spec.Explode)
    case []int:
        return SerializePathArray(spec.Name, intSliceToInterface(typed), style, spec.Explode)
    case []interface{}:
        return SerializePathArray(spec.Name, typed, style, spec.Explode)
    case map[string]string:
        return SerializePathObject(spec.Name, stringMapToInterface(typed), style, spec.Explode)
    case map[string]int:
        return SerializePathObject(spec.Name, intMapToInterface(typed), style, spec.Explode)
    case map[string]interface{}:
        return SerializePathObject(spec.Name, typed, style, spec.Explode)
    default:
        return PathPrefix(spec.Name, style) + url.PathEscape(fmt.Sprint(value))
    }
}

func SerializePathArray(name string, values []interface{}, style string, explode bool) string {
    serialized := make([]string, 0, len(values))
    for _, item := range values {
        if item != nil {
            serialized = append(serialized, url.PathEscape(fmt.Sprint(item)))
        }
    }
    if len(serialized) == 0 {
        return PathPrefix(name, style)
    }
    if style == "matrix" {
        if explode {
            parts := make([]string, 0, len(serialized))
            for _, item := range serialized {
                parts = append(parts, ";"+name+"="+item)
            }
            return strings.Join(parts, "")
        }
        return ";" + name + "=" + strings.Join(serialized, ",")
    }
    separator := ","
    if explode {
        separator = "."
    }
    return PathPrefix(name, style) + strings.Join(serialized, separator)
}

func SerializePathObject(name string, values map[string]interface{}, style string, explode bool) string {
    entries := make([]string, 0, len(values)*2)
    exploded := make([]string, 0, len(values))
    for key, value := range values {
        if value == nil {
            continue
        }
        escapedKey := url.PathEscape(key)
        escapedValue := url.PathEscape(fmt.Sprint(value))
        if explode {
            if style == "matrix" {
                exploded = append(exploded, ";"+escapedKey+"="+escapedValue)
            } else {
                exploded = append(exploded, escapedKey+"="+escapedValue)
            }
        } else {
            entries = append(entries, escapedKey, escapedValue)
        }
    }
    if style == "matrix" {
        if explode {
            return strings.Join(exploded, "")
        }
        return ";" + name + "=" + strings.Join(entries, ",")
    }
    if explode {
        separator := ","
        if style == "label" {
            separator = "."
        }
        return PathPrefix(name, style) + strings.Join(exploded, separator)
    }
    return PathPrefix(name, style) + strings.Join(entries, ",")
}

func PathPrefix(name string, style string) string {
    if style == "label" {
        return "."
    }
    if style == "matrix" {
        return ";" + name
    }
    return ""
}
type QueryParameterSpec struct {
    Name          string
    Value         interface{}
    Style         string
    Explode       bool
    AllowReserved bool
    ContentType   string
}

func BuildQueryString(parameters []QueryParameterSpec) string {
    pairs := make([]string, 0)
    for _, parameter := range parameters {
        AppendSerializedParameter(&pairs, parameter)
    }
    return strings.Join(pairs, "&")
}

func AppendSerializedParameter(pairs *[]string, parameter QueryParameterSpec) {
    if parameter.Value == nil {
        return
    }

    if parameter.ContentType != "" {
        encoded, _ := json.Marshal(parameter.Value)
        *pairs = append(*pairs, url.QueryEscape(parameter.Name)+"="+EncodeQueryValue(string(encoded), parameter.AllowReserved))
        return
    }

    style := parameter.Style
    if style == "" {
        style = "form"
    }

    switch value := parameter.Value.(type) {
    case []string:
        AppendArrayParameter(pairs, parameter.Name, stringSliceToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case []int:
        AppendArrayParameter(pairs, parameter.Name, intSliceToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case []interface{}:
        AppendArrayParameter(pairs, parameter.Name, value, style, parameter.Explode, parameter.AllowReserved)
    case map[string]int:
        AppendObjectParameter(pairs, parameter.Name, intMapToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case map[string]string:
        AppendObjectParameter(pairs, parameter.Name, stringMapToInterface(value), style, parameter.Explode, parameter.AllowReserved)
    case map[string]interface{}:
        if style == "deepObject" {
            AppendDeepObjectParameter(pairs, parameter.Name, value, parameter.AllowReserved)
        } else {
            AppendObjectParameter(pairs, parameter.Name, value, style, parameter.Explode, parameter.AllowReserved)
        }
    default:
        *pairs = append(*pairs, url.QueryEscape(parameter.Name)+"="+EncodeQueryValue(fmt.Sprint(value), parameter.AllowReserved))
    }
}

func AppendArrayParameter(pairs *[]string, name string, value []interface{}, style string, explode bool, allowReserved bool) {
    values := make([]string, 0, len(value))
    for _, item := range value {
        if item != nil {
            values = append(values, fmt.Sprint(item))
        }
    }
    if len(values) == 0 {
        return
    }
    if style == "form" && explode {
        for _, item := range values {
            *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(item, allowReserved))
        }
        return
    }
    *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(strings.Join(values, ","), allowReserved))
}

func AppendObjectParameter(pairs *[]string, name string, value map[string]interface{}, style string, explode bool, allowReserved bool) {
    entries := make([]string, 0, len(value)*2)
    for key, item := range value {
        if item == nil {
            continue
        }
        if style == "form" && explode {
            *pairs = append(*pairs, url.QueryEscape(key)+"="+EncodeQueryValue(fmt.Sprint(item), allowReserved))
            continue
        }
        entries = append(entries, key, fmt.Sprint(item))
    }
    if len(entries) == 0 {
        return
    }
    if !(style == "form" && explode) {
        *pairs = append(*pairs, url.QueryEscape(name)+"="+EncodeQueryValue(strings.Join(entries, ","), allowReserved))
    }
}

func AppendDeepObjectParameter(pairs *[]string, name string, value map[string]interface{}, allowReserved bool) {
    for key, item := range value {
        if item == nil {
            continue
        }
        *pairs = append(*pairs, url.QueryEscape(fmt.Sprintf("%s[%s]", name, key))+"="+EncodeQueryValue(fmt.Sprint(item), allowReserved))
    }
}

func EncodeQueryValue(value string, allowReserved bool) string {
    encoded := url.QueryEscape(value)
    if !allowReserved {
        return encoded
    }
    replacements := map[string]string{
        "%3A": ":", "%2F": "/", "%3F": "?", "%23": "#",
        "%5B": "[", "%5D": "]", "%40": "@", "%21": "!",
        "%24": "$", "%26": "&", "%27": "'", "%28": "(",
        "%29": ")", "%2A": "*", "%2B": "+", "%2C": ",",
        "%3B": ";", "%3D": "=",
    }
    for escaped, reserved := range replacements {
        encoded = strings.ReplaceAll(encoded, escaped, reserved)
    }
    return encoded
}



func stringSliceToInterface(values []string) []interface{} {
    result := make([]interface{}, 0, len(values))
    for _, value := range values {
        result = append(result, value)
    }
    return result
}

func intSliceToInterface(values []int) []interface{} {
    result := make([]interface{}, 0, len(values))
    for _, value := range values {
        result = append(result, value)
    }
    return result
}

func stringMapToInterface(values map[string]string) map[string]interface{} {
    result := make(map[string]interface{}, len(values))
    for key, value := range values {
        result[key] = value
    }
    return result
}

func intMapToInterface(values map[string]int) map[string]interface{} {
    result := make(map[string]interface{}, len(values))
    for key, value := range values {
        result[key] = value
    }
    return result
}
