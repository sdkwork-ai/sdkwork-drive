package api

import (
    "encoding/json"
    "fmt"
    "net/url"
    "strings"
    sdktypes "sdkwork-drive-app-sdk-generated-go/types"
    sdkhttp "sdkwork-drive-app-sdk-generated-go/http"
)

type DriveApi struct {
    client *sdkhttp.Client
}

func NewDriveApi(client *sdkhttp.Client) *DriveApi {
    return &DriveApi{client: client}
}

func (a *DriveApi) ChangesList(spaceId string, cursor *int, pageSize *int) (sdktypes.ChangeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: spaceId, Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/changes"), query), nil, nil)
    if err != nil {
        var zero sdktypes.ChangeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ChangeListHttpResponse](raw)
}

func (a *DriveApi) ChangesStartPageTokenRetrieve(spaceId string) (sdktypes.StartPageTokenHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: spaceId, Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/changes/start_page_token"), query), nil, nil)
    if err != nil {
        var zero sdktypes.StartPageTokenHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.StartPageTokenHttpResponse](raw)
}

func (a *DriveApi) DownloadTokensRetrieve(token string) (sdktypes.CreateDownloadUrlHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/download_tokens/%s", SerializePathParameter(token, PathParameterSpec{Name: "token", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.CreateDownloadUrlHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateDownloadUrlHttpResponse](raw)
}

func (a *DriveApi) DownloadUrlsCreate(body sdktypes.CreateDownloadUrlRequest) (sdktypes.CreateDownloadUrlHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/download_urls"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.CreateDownloadUrlHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateDownloadUrlHttpResponse](raw)
}

func (a *DriveApi) FavoritesList(spaceId *string, pageSize *int, cursor *string, sortBy *string, sortOrder *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: func() interface{} { if spaceId == nil { return nil }; return *spaceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortBy", Value: func() interface{} { if sortBy == nil { return nil }; return *sortBy }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortOrder", Value: func() interface{} { if sortOrder == nil { return nil }; return *sortOrder }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/favorites"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) FavoritesCheck(body sdktypes.CheckFavoriteNodesRequest) (sdktypes.SdkWorkApiResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/favorites/check"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.SdkWorkApiResponse
        return zero, err
    }
    return decodeResult[sdktypes.SdkWorkApiResponse](raw)
}

func (a *DriveApi) QuotasRetrieve() (sdktypes.QuotaSummaryHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath("/drive/quotas/summary"), nil, nil)
    if err != nil {
        var zero sdktypes.QuotaSummaryHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.QuotaSummaryHttpResponse](raw)
}

func (a *DriveApi) NodesUpdate(nodeId string, body sdktypes.UpdateNodeRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/nodes/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) NodesRetrieve(nodeId string) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) NodesDelete(nodeId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) NodesCapabilitiesList(nodeId string) (sdktypes.NodeCapabilitiesHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/capabilities", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.NodeCapabilitiesHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.NodeCapabilitiesHttpResponse](raw)
}

func (a *DriveApi) CommentsList(nodeId string, pageSize *int, cursor *string) (sdktypes.DriveCommentListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveCommentListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentListHttpResponse](raw)
}

func (a *DriveApi) CommentsCreate(nodeId string, body sdktypes.CreateCommentRequest) (sdktypes.DriveCommentHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveCommentHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentHttpResponse](raw)
}

func (a *DriveApi) CommentsRetrieve(nodeId string, commentId string) (sdktypes.DriveCommentHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveCommentHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentHttpResponse](raw)
}

func (a *DriveApi) CommentsUpdate(nodeId string, commentId string, body sdktypes.UpdateCommentRequest) (sdktypes.DriveCommentHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveCommentHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentHttpResponse](raw)
}

func (a *DriveApi) CommentsDelete(nodeId string, commentId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) CommentRepliesList(nodeId string, commentId string, pageSize *int, cursor *string) (sdktypes.DriveCommentReplyListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s/replies", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveCommentReplyListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentReplyListHttpResponse](raw)
}

func (a *DriveApi) CommentRepliesCreate(nodeId string, commentId string, body sdktypes.CreateCommentReplyRequest) (sdktypes.DriveCommentReplyHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s/replies", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveCommentReplyHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentReplyHttpResponse](raw)
}

func (a *DriveApi) CommentRepliesRetrieve(nodeId string, commentId string, replyId string) (sdktypes.DriveCommentReplyHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s/replies/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}), SerializePathParameter(replyId, PathParameterSpec{Name: "replyId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveCommentReplyHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentReplyHttpResponse](raw)
}

func (a *DriveApi) CommentRepliesUpdate(nodeId string, commentId string, replyId string, body sdktypes.UpdateCommentReplyRequest) (sdktypes.DriveCommentReplyHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s/replies/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}), SerializePathParameter(replyId, PathParameterSpec{Name: "replyId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveCommentReplyHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveCommentReplyHttpResponse](raw)
}

func (a *DriveApi) CommentRepliesDelete(nodeId string, commentId string, replyId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/comments/%s/replies/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(commentId, PathParameterSpec{Name: "commentId", Style: "simple", Explode: false}), SerializePathParameter(replyId, PathParameterSpec{Name: "replyId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) NodesCopy(nodeId string, body sdktypes.CopyNodeRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/copy", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) NodesDownloadUrlsRetrieve(nodeId string, requestedTtlSeconds *int) (sdktypes.CreateDownloadUrlHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "requestedTtlSeconds", Value: func() interface{} { if requestedTtlSeconds == nil { return nil }; return *requestedTtlSeconds }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/download_url", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.CreateDownloadUrlHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateDownloadUrlHttpResponse](raw)
}

func (a *DriveApi) DownloadGrantsCreate(nodeId string, body *sdktypes.CreateDownloadGrantRequest) (sdktypes.CreateDownloadUrlHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/download_grants", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.CreateDownloadUrlHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateDownloadUrlHttpResponse](raw)
}

func (a *DriveApi) FavoritesUpdate(nodeId string, body sdktypes.FavoriteNodeRequest) (sdktypes.FavoriteNodeHttpResponse, error) {
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/nodes/%s/favorite", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.FavoriteNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.FavoriteNodeHttpResponse](raw)
}

func (a *DriveApi) FavoritesDelete(nodeId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/favorite", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

// List labels applied to a node
func (a *DriveApi) NodeLabelsList(nodeId string, labelKey *string, pageSize *int, cursor *string) (sdktypes.NodeLabelListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "labelKey", Value: func() interface{} { if labelKey == nil { return nil }; return *labelKey }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/labels", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.NodeLabelListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.NodeLabelListHttpResponse](raw)
}

// Apply a label to a node
func (a *DriveApi) NodeLabelsUpdate(nodeId string, labelId string, body sdktypes.ApplyNodeLabelRequest) (sdktypes.NodeLabelHttpResponse, error) {
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/nodes/%s/labels/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(labelId, PathParameterSpec{Name: "labelId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.NodeLabelHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.NodeLabelHttpResponse](raw)
}

// Remove a label from a node
func (a *DriveApi) NodeLabelsDelete(nodeId string, labelId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/labels/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(labelId, PathParameterSpec{Name: "labelId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) NodesMove(nodeId string, body sdktypes.MoveNodeRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/move", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) NodesPathRetrieve(nodeId string) (sdktypes.NodePathHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/path", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.NodePathHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.NodePathHttpResponse](raw)
}

func (a *DriveApi) PermissionsList(nodeId string, pageSize *int, cursor *string) (sdktypes.DrivePermissionListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DrivePermissionListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DrivePermissionListHttpResponse](raw)
}

func (a *DriveApi) PermissionsCreate(nodeId string, body sdktypes.CreatePermissionRequest) (sdktypes.DrivePermissionHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DrivePermissionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DrivePermissionHttpResponse](raw)
}

func (a *DriveApi) PermissionsDelete(nodeId string, permissionId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(permissionId, PathParameterSpec{Name: "permissionId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) PermissionsUpdate(nodeId string, permissionId string, body sdktypes.UpdatePermissionRequest) (sdktypes.DrivePermissionHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(permissionId, PathParameterSpec{Name: "permissionId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DrivePermissionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DrivePermissionHttpResponse](raw)
}

func (a *DriveApi) PermissionsRetrieve(nodeId string, permissionId string) (sdktypes.DrivePermissionHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(permissionId, PathParameterSpec{Name: "permissionId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DrivePermissionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DrivePermissionHttpResponse](raw)
}

func (a *DriveApi) PermissionsEffectiveList(nodeId string, pageSize *int, cursor *string) (sdktypes.EffectivePermissionListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/permissions/effective", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.EffectivePermissionListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.EffectivePermissionListHttpResponse](raw)
}

// List node custom properties
func (a *DriveApi) NodePropertiesList(nodeId string, visibility *string, pageSize *int, cursor *string) (sdktypes.DriveNodePropertyListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "visibility", Value: func() interface{} { if visibility == nil { return nil }; return *visibility }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/properties", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodePropertyListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodePropertyListHttpResponse](raw)
}

// Create or update a node custom property
func (a *DriveApi) NodePropertiesUpdate(nodeId string, propertyKey string, body sdktypes.SetNodePropertyRequest) (sdktypes.DriveNodePropertyHttpResponse, error) {
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/nodes/%s/properties/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(propertyKey, PathParameterSpec{Name: "propertyKey", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodePropertyHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodePropertyHttpResponse](raw)
}

// Delete a node custom property
func (a *DriveApi) NodePropertiesDelete(nodeId string, propertyKey string, visibility *string) (struct{}, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "visibility", Value: func() interface{} { if visibility == nil { return nil }; return *visibility }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Delete(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/properties/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(propertyKey, PathParameterSpec{Name: "propertyKey", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) ShareLinksCreate(nodeId string, body sdktypes.CreateShareLinkRequest) (sdktypes.CreateShareLinkHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/share_links", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.CreateShareLinkHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateShareLinkHttpResponse](raw)
}

func (a *DriveApi) ShareLinksList(nodeId string, pageSize *int, cursor *string) (sdktypes.ShareLinkListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/share_links", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.ShareLinkListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ShareLinkListHttpResponse](raw)
}

func (a *DriveApi) TrashCreate(nodeId string, body sdktypes.NodeCommandRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/trash", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) VersionsList(nodeId string, pageSize *int, cursor *string) (sdktypes.FileVersionListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/nodes/%s/versions", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.FileVersionListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.FileVersionListHttpResponse](raw)
}

func (a *DriveApi) VersionsDelete(nodeId string, versionId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/nodes/%s/versions/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(versionId, PathParameterSpec{Name: "versionId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) VersionsRetrieve(nodeId string, versionId string) (sdktypes.FileVersionHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/versions/%s", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(versionId, PathParameterSpec{Name: "versionId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.FileVersionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.FileVersionHttpResponse](raw)
}

func (a *DriveApi) VersionsRestore(nodeId string, versionId string, body sdktypes.NodeCommandRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/versions/%s/restore", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}), SerializePathParameter(versionId, PathParameterSpec{Name: "versionId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) NodesFilesCreate(body sdktypes.CreateFileRequest) (sdktypes.CreateFileHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/nodes/files"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.CreateFileHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.CreateFileHttpResponse](raw)
}

func (a *DriveApi) NodesFoldersCreate(body sdktypes.CreateFolderRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/nodes/folders"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

// Create a shortcut node
func (a *DriveApi) NodesShortcutsCreate(body sdktypes.CreateShortcutRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/nodes/shortcuts"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

// List nodes carrying an app_public property
func (a *DriveApi) PropertyNodesList(propertyKey string, pageSize *int, cursor *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/properties/%s/nodes", SerializePathParameter(propertyKey, PathParameterSpec{Name: "propertyKey", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) RecentList(spaceId *string, pageSize *int, cursor *string, sortBy *string, sortOrder *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: func() interface{} { if spaceId == nil { return nil }; return *spaceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortBy", Value: func() interface{} { if sortBy == nil { return nil }; return *sortBy }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortOrder", Value: func() interface{} { if sortOrder == nil { return nil }; return *sortOrder }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/recent"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) SearchList(q *string, spaceId *string, pageSize *int, cursor *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "q", Value: func() interface{} { if q == nil { return nil }; return *q }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "spaceId", Value: func() interface{} { if spaceId == nil { return nil }; return *spaceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/search"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) ShareLinksClaim(token string) (sdktypes.ClaimShareLinkHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/share_links/%s/claim", SerializePathParameter(token, PathParameterSpec{Name: "token", Style: "simple", Explode: false}))), nil, nil, nil, "")
    if err != nil {
        var zero sdktypes.ClaimShareLinkHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ClaimShareLinkHttpResponse](raw)
}

func (a *DriveApi) ShareLinksDelete(shareLinkId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/share_links/%s", SerializePathParameter(shareLinkId, PathParameterSpec{Name: "shareLinkId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) ShareLinksUpdate(shareLinkId string, body sdktypes.UpdateShareLinkRequest) (sdktypes.ShareLinkHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/share_links/%s", SerializePathParameter(shareLinkId, PathParameterSpec{Name: "shareLinkId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.ShareLinkHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ShareLinkHttpResponse](raw)
}

func (a *DriveApi) ShareLinksRetrieve(shareLinkId string) (sdktypes.ShareLinkHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/share_links/%s", SerializePathParameter(shareLinkId, PathParameterSpec{Name: "shareLinkId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.ShareLinkHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ShareLinkHttpResponse](raw)
}

func (a *DriveApi) SharedWithMeList(spaceId *string, pageSize *int, cursor *string, sortBy *string, sortOrder *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: func() interface{} { if spaceId == nil { return nil }; return *spaceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortBy", Value: func() interface{} { if sortBy == nil { return nil }; return *sortBy }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortOrder", Value: func() interface{} { if sortOrder == nil { return nil }; return *sortOrder }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/shared_with_me"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) SandboxesList(page *int, pageSize *int) (sdktypes.DriveSandboxVolumeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page", Value: func() interface{} { if page == nil { return nil }; return *page }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/sandboxes"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveSandboxVolumeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxVolumeListHttpResponse](raw)
}

func (a *DriveApi) SandboxEntriesList(sandboxId string, parentPath *string, cursor *string, pageSize *int) (sdktypes.DriveSandboxEntryListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "parent_path", Value: func() interface{} { if parentPath == nil { return nil }; return *parentPath }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/entries", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveSandboxEntryListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxEntryListHttpResponse](raw)
}

func (a *DriveApi) SandboxDirectoriesCreate(sandboxId string, body sdktypes.CreateDriveSandboxDirectoryRequest, idempotencyKey string) (sdktypes.DriveSandboxEntryHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{"Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},},
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/directories", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.DriveSandboxEntryHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxEntryHttpResponse](raw)
}

func (a *DriveApi) SandboxFilesCreate(sandboxId string, body sdktypes.CreateDriveSandboxFileRequest, idempotencyKey string) (sdktypes.DriveSandboxEntryHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{"Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},},
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/files", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.DriveSandboxEntryHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxEntryHttpResponse](raw)
}

func (a *DriveApi) SandboxFileContentsRetrieve(sandboxId string, entryId string, logicalPath string, encoding *string) (sdktypes.DriveSandboxFileContentHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "logical_path", Value: logicalPath, Style: "form", Explode: true, AllowReserved: false},
        {Name: "encoding", Value: func() interface{} { if encoding == nil { return nil }; return *encoding }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/files/%s/content", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(entryId, PathParameterSpec{Name: "entryId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveSandboxFileContentHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxFileContentHttpResponse](raw)
}

func (a *DriveApi) SandboxFileContentsUpdate(sandboxId string, entryId string, body sdktypes.UpdateDriveSandboxFileContentRequest, ifMatch string, idempotencyKey string) (sdktypes.DriveSandboxEntryHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{
            "If-Match": ParameterSpec{Value: ifMatch, Style: "simple", Explode: false},
            "Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},
        },
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/files/%s/content", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(entryId, PathParameterSpec{Name: "entryId", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.DriveSandboxEntryHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxEntryHttpResponse](raw)
}

func (a *DriveApi) SandboxEntriesUpdate(sandboxId string, entryId string, body sdktypes.UpdateDriveSandboxEntryRequest, ifMatch string, idempotencyKey string) (sdktypes.DriveSandboxEntryHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{
            "If-Match": ParameterSpec{Value: ifMatch, Style: "simple", Explode: false},
            "Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},
        },
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/entries/%s", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(entryId, PathParameterSpec{Name: "entryId", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.DriveSandboxEntryHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxEntryHttpResponse](raw)
}

func (a *DriveApi) SandboxEntriesPurge(sandboxId string, entryId string, body sdktypes.PurgeDriveSandboxEntryRequest, ifMatch string, idempotencyKey string) (sdktypes.DriveSandboxMutationCommandHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{
            "If-Match": ParameterSpec{Value: ifMatch, Style: "simple", Explode: false},
            "Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},
        },
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/sandboxes/%s/entries/%s/purge", SerializePathParameter(sandboxId, PathParameterSpec{Name: "sandboxId", Style: "simple", Explode: false}), SerializePathParameter(entryId, PathParameterSpec{Name: "entryId", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.DriveSandboxMutationCommandHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSandboxMutationCommandHttpResponse](raw)
}

func (a *DriveApi) SpacesList(ownerSubjectType *string, ownerSubjectId *string, spaceType *string, pageSize *int, cursor *string) (sdktypes.DriveSpaceListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "ownerSubjectType", Value: func() interface{} { if ownerSubjectType == nil { return nil }; return *ownerSubjectType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "ownerSubjectId", Value: func() interface{} { if ownerSubjectId == nil { return nil }; return *ownerSubjectId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "spaceType", Value: func() interface{} { if spaceType == nil { return nil }; return *spaceType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/spaces"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveSpaceListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSpaceListHttpResponse](raw)
}

func (a *DriveApi) SpacesCreate(body sdktypes.CreateSpaceRequest) (sdktypes.DriveSpaceHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/spaces"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveSpaceHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSpaceHttpResponse](raw)
}

func (a *DriveApi) WebsiteRootsList(spaceId string, pageSize *int, cursor *string) (sdktypes.WebsiteRootListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/spaces/%s/website_roots", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.WebsiteRootListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteRootListHttpResponse](raw)
}

func (a *DriveApi) WebsiteRootsCreate(spaceId string, body sdktypes.CreateWebsiteRootRequest) (sdktypes.WebsiteRootHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/spaces/%s/website_roots", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.WebsiteRootHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteRootHttpResponse](raw)
}

func (a *DriveApi) WebsiteRootsRetrieve(rootUuid string) (sdktypes.WebsiteRootHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/website_roots/%s", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.WebsiteRootHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteRootHttpResponse](raw)
}

// Create an isolated atomic website synchronization
func (a *DriveApi) WebsiteRootsSyncsCreate(rootUuid string, body sdktypes.CreateWebsiteSyncRequest, idempotencyKey string) (sdktypes.WebsiteSyncHttpResponse, error) {
    headers := BuildRequestHeaders(
        map[string]ParameterSpec{"Idempotency-Key": ParameterSpec{Value: idempotencyKey, Style: "simple", Explode: false},},
        map[string]ParameterSpec{},
    )
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/website_roots/%s/syncs", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}))), body, nil, headers, "application/json")
    if err != nil {
        var zero sdktypes.WebsiteSyncHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteSyncHttpResponse](raw)
}

// Retrieve an atomic website synchronization
func (a *DriveApi) WebsiteRootsSyncsRetrieve(rootUuid string, syncId string) (sdktypes.WebsiteSyncHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/website_roots/%s/syncs/%s", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}), SerializePathParameter(syncId, PathParameterSpec{Name: "syncId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.WebsiteSyncHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteSyncHttpResponse](raw)
}

// Validate and atomically activate a complete website tree
func (a *DriveApi) WebsiteRootsSyncsFinalize(rootUuid string, syncId string, body sdktypes.WebsiteSyncVersionRequest) (sdktypes.WebsiteSyncActivationHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/website_roots/%s/syncs/%s/finalize", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}), SerializePathParameter(syncId, PathParameterSpec{Name: "syncId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.WebsiteSyncActivationHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteSyncActivationHttpResponse](raw)
}

// Abort an unactivated website synchronization
func (a *DriveApi) WebsiteRootsSyncsAbort(rootUuid string, syncId string, body sdktypes.WebsiteSyncVersionRequest) (sdktypes.WebsiteSyncHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/website_roots/%s/syncs/%s/abort", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}), SerializePathParameter(syncId, PathParameterSpec{Name: "syncId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.WebsiteSyncHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteSyncHttpResponse](raw)
}

// Activate a retained website generation as a new logical generation
func (a *DriveApi) WebsiteRootsGenerationsActivate(rootUuid string, generation sdktypes.PositiveInt64String, body sdktypes.ActivateWebsiteGenerationRequest) (sdktypes.WebsiteGenerationActivationHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/website_roots/%s/generations/%s/activate", SerializePathParameter(rootUuid, PathParameterSpec{Name: "rootUuid", Style: "simple", Explode: false}), SerializePathParameter(generation, PathParameterSpec{Name: "generation", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.WebsiteGenerationActivationHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.WebsiteGenerationActivationHttpResponse](raw)
}

func (a *DriveApi) MoveDestinationsList(spaceId string, excludeNodeIds *string, pageSize *int, cursor *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "excludeNodeIds", Value: func() interface{} { if excludeNodeIds == nil { return nil }; return *excludeNodeIds }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/spaces/%s/move_destinations", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) SpacesRetrieve(spaceId string) (sdktypes.DriveSpaceHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/spaces/%s", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveSpaceHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSpaceHttpResponse](raw)
}

func (a *DriveApi) SpacesUpdate(spaceId string, body sdktypes.UpdateSpaceRequest) (sdktypes.DriveSpaceHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/drive/spaces/%s", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveSpaceHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveSpaceHttpResponse](raw)
}

func (a *DriveApi) SpacesDelete(spaceId string) (struct{}, error) {
    raw, err := a.client.Delete(AppApiPath(fmt.Sprintf("/drive/spaces/%s", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero struct{}
        return zero, err
    }
    return decodeResult[struct{}](raw)
}

func (a *DriveApi) NodesList(spaceId string, parentNodeId *string, pageSize *int, cursor *string, sortBy *string, sortOrder *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "parentNodeId", Value: func() interface{} { if parentNodeId == nil { return nil }; return *parentNodeId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortBy", Value: func() interface{} { if sortBy == nil { return nil }; return *sortBy }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortOrder", Value: func() interface{} { if sortOrder == nil { return nil }; return *sortOrder }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath(fmt.Sprintf("/drive/spaces/%s/nodes", SerializePathParameter(spaceId, PathParameterSpec{Name: "spaceId", Style: "simple", Explode: false}))), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) TrashList(spaceId *string, pageSize *int, cursor *string, parentNodeId *string, sortBy *string, sortOrder *string) (sdktypes.DriveNodeListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "spaceId", Value: func() interface{} { if spaceId == nil { return nil }; return *spaceId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "parentNodeId", Value: func() interface{} { if parentNodeId == nil { return nil }; return *parentNodeId }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortBy", Value: func() interface{} { if sortBy == nil { return nil }; return *sortBy }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sortOrder", Value: func() interface{} { if sortOrder == nil { return nil }; return *sortOrder }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/trash"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveNodeListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeListHttpResponse](raw)
}

func (a *DriveApi) TrashRestore(nodeId string, body sdktypes.NodeCommandRequest) (sdktypes.DriveNodeHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/trash/%s/restore", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveNodeHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveNodeHttpResponse](raw)
}

func (a *DriveApi) TrashEmpty(body sdktypes.EmptyTrashRequest) (sdktypes.EmptyTrashHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/trash/empty"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.EmptyTrashHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.EmptyTrashHttpResponse](raw)
}

func (a *DriveApi) UploadSessionsCreate(body sdktypes.CreateUploadSessionRequest) (sdktypes.DriveUploadSessionHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/upload_sessions"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveUploadSessionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveUploadSessionHttpResponse](raw)
}

func (a *DriveApi) UploadSessionsRetrieve(uploadSessionId string) (sdktypes.DriveUploadSessionHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/upload_sessions/%s", SerializePathParameter(uploadSessionId, PathParameterSpec{Name: "uploadSessionId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveUploadSessionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveUploadSessionHttpResponse](raw)
}

func (a *DriveApi) UploadSessionsAbort(uploadSessionId string, body sdktypes.NodeCommandRequest) (sdktypes.DriveUploadSessionHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/upload_sessions/%s/abort", SerializePathParameter(uploadSessionId, PathParameterSpec{Name: "uploadSessionId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveUploadSessionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveUploadSessionHttpResponse](raw)
}

func (a *DriveApi) UploadSessionsComplete(uploadSessionId string, body sdktypes.CompleteUploadSessionRequest) (sdktypes.DriveUploadSessionHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/upload_sessions/%s/complete", SerializePathParameter(uploadSessionId, PathParameterSpec{Name: "uploadSessionId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveUploadSessionHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveUploadSessionHttpResponse](raw)
}

func (a *DriveApi) UploadSessionsPartsUpdate(uploadSessionId string, partNo int, body sdktypes.PresignUploadPartRequest) (sdktypes.PresignedUploadPartHttpResponse, error) {
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/upload_sessions/%s/parts/%s", SerializePathParameter(uploadSessionId, PathParameterSpec{Name: "uploadSessionId", Style: "simple", Explode: false}), SerializePathParameter(partNo, PathParameterSpec{Name: "partNo", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.PresignedUploadPartHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.PresignedUploadPartHttpResponse](raw)
}

// Create a push notification channel for Drive changes
func (a *DriveApi) ChangesWatch(body sdktypes.CreateWatchChannelRequest) (sdktypes.DriveWatchChannelHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/changes/watch"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveWatchChannelHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveWatchChannelHttpResponse](raw)
}

// Create a push notification channel for a Drive node
func (a *DriveApi) NodesWatch(nodeId string, body sdktypes.CreateWatchChannelRequest) (sdktypes.DriveWatchChannelHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/watch", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DriveWatchChannelHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveWatchChannelHttpResponse](raw)
}

// List Drive watch channels
func (a *DriveApi) WatchChannelsList(resourceType *string, lifecycleStatus *string, pageSize *int, cursor *string) (sdktypes.DriveWatchChannelListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "resourceType", Value: func() interface{} { if resourceType == nil { return nil }; return *resourceType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "lifecycleStatus", Value: func() interface{} { if lifecycleStatus == nil { return nil }; return *lifecycleStatus }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/drive/watch_channels"), query), nil, nil)
    if err != nil {
        var zero sdktypes.DriveWatchChannelListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveWatchChannelListHttpResponse](raw)
}

// Get a Drive watch channel
func (a *DriveApi) WatchChannelsRetrieve(channelId string) (sdktypes.DriveWatchChannelHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/watch_channels/%s", SerializePathParameter(channelId, PathParameterSpec{Name: "channelId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DriveWatchChannelHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DriveWatchChannelHttpResponse](raw)
}

// Stop a Drive watch channel
func (a *DriveApi) WatchChannelsStop(channelId string, body sdktypes.StopWatchChannelRequest) (sdktypes.StopWatchChannelHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/watch_channels/%s/stop", SerializePathParameter(channelId, PathParameterSpec{Name: "channelId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.StopWatchChannelHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.StopWatchChannelHttpResponse](raw)
}

func (a *DriveApi) DownloadPackagesCreate(body sdktypes.CreateDownloadPackageRequest) (sdktypes.DownloadPackageHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/download_packages"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.DownloadPackageHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DownloadPackageHttpResponse](raw)
}

func (a *DriveApi) DownloadPackagesUrlsRetrieve(packageId string) (sdktypes.DownloadPackageHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/download_packages/%s/download_url", SerializePathParameter(packageId, PathParameterSpec{Name: "packageId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.DownloadPackageHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.DownloadPackageHttpResponse](raw)
}

func (a *DriveApi) ArchiveEntriesList(nodeId string) (sdktypes.ArchiveEntryListHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/drive/nodes/%s/archive_entries", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.ArchiveEntryListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ArchiveEntryListHttpResponse](raw)
}

func (a *DriveApi) ArchiveEntriesExtract(nodeId string, body sdktypes.ExtractArchiveEntriesRequest) (sdktypes.ExtractArchiveEntriesHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/drive/nodes/%s/archive_entries/extract", SerializePathParameter(nodeId, PathParameterSpec{Name: "nodeId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.ExtractArchiveEntriesHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.ExtractArchiveEntriesHttpResponse](raw)
}

func (a *DriveApi) UploaderUploadsCreate(body sdktypes.PrepareUploaderUploadRequest) (sdktypes.PrepareUploaderUploadHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/drive/uploader/uploads"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.PrepareUploaderUploadHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.PrepareUploaderUploadHttpResponse](raw)
}

func (a *DriveApi) UploaderUploadsPartsUpdate(uploadItemId string, partNo int, body sdktypes.MarkUploaderPartUploadedRequest) (sdktypes.UploaderUploadPartHttpResponse, error) {
    raw, err := a.client.Put(AppApiPath(fmt.Sprintf("/drive/uploader/uploads/%s/parts/%s", SerializePathParameter(uploadItemId, PathParameterSpec{Name: "uploadItemId", Style: "simple", Explode: false}), SerializePathParameter(partNo, PathParameterSpec{Name: "partNo", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.UploaderUploadPartHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.UploaderUploadPartHttpResponse](raw)
}

// List global assets
func (a *DriveApi) AssetsList(cursor *string, pageSize *int, kind *string, sourceType *string, q *string) (sdktypes.AssetListHttpResponse, error) {
    query := BuildQueryString([]QueryParameterSpec{
        {Name: "cursor", Value: func() interface{} { if cursor == nil { return nil }; return *cursor }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "page_size", Value: func() interface{} { if pageSize == nil { return nil }; return *pageSize }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "kind", Value: func() interface{} { if kind == nil { return nil }; return *kind }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "sourceType", Value: func() interface{} { if sourceType == nil { return nil }; return *sourceType }(), Style: "form", Explode: true, AllowReserved: false},
        {Name: "q", Value: func() interface{} { if q == nil { return nil }; return *q }(), Style: "form", Explode: true, AllowReserved: false},
    })
    raw, err := a.client.Get(AppendQueryString(AppApiPath("/assets"), query), nil, nil)
    if err != nil {
        var zero sdktypes.AssetListHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetListHttpResponse](raw)
}

// Create a global asset metadata record
func (a *DriveApi) AssetsCreate(body sdktypes.CreateAssetRequest) (sdktypes.AssetItemHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath("/assets"), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.AssetItemHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetItemHttpResponse](raw)
}

// Get a global asset
func (a *DriveApi) AssetsRetrieve(assetId string) (sdktypes.AssetItemHttpResponse, error) {
    raw, err := a.client.Get(AppApiPath(fmt.Sprintf("/assets/%s", SerializePathParameter(assetId, PathParameterSpec{Name: "assetId", Style: "simple", Explode: false}))), nil, nil)
    if err != nil {
        var zero sdktypes.AssetItemHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetItemHttpResponse](raw)
}

// Update a global asset
func (a *DriveApi) AssetsUpdate(assetId string, body sdktypes.UpdateAssetRequest) (sdktypes.AssetItemHttpResponse, error) {
    raw, err := a.client.Patch(AppApiPath(fmt.Sprintf("/assets/%s", SerializePathParameter(assetId, PathParameterSpec{Name: "assetId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.AssetItemHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetItemHttpResponse](raw)
}

// Archive a global asset
func (a *DriveApi) AssetsArchive(assetId string, body sdktypes.AssetActionRequest) (sdktypes.AssetItemHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/assets/%s/archive", SerializePathParameter(assetId, PathParameterSpec{Name: "assetId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.AssetItemHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetItemHttpResponse](raw)
}

// Restore an archived global asset
func (a *DriveApi) AssetsRestore(assetId string, body sdktypes.AssetActionRequest) (sdktypes.AssetItemHttpResponse, error) {
    raw, err := a.client.Post(AppApiPath(fmt.Sprintf("/assets/%s/restore", SerializePathParameter(assetId, PathParameterSpec{Name: "assetId", Style: "simple", Explode: false}))), body, nil, nil, "application/json")
    if err != nil {
        var zero sdktypes.AssetItemHttpResponse
        return zero, err
    }
    return decodeResult[sdktypes.AssetItemHttpResponse](raw)
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


type ParameterSpec struct {
    Value       interface{}
    Style       string
    Explode     bool
    ContentType string
}

func BuildRequestHeaders(headers map[string]ParameterSpec, cookies map[string]ParameterSpec) map[string]string {
    requestHeaders := map[string]string{}
    for name, parameter := range headers {
        if serialized, ok := SerializeParameterValue(parameter); ok {
            requestHeaders[name] = serialized
        }
    }

    if cookieHeader := BuildCookieHeader(cookies); cookieHeader != "" {
        if existing, ok := requestHeaders["Cookie"]; ok && existing != "" {
            requestHeaders["Cookie"] = existing + "; " + cookieHeader
        } else {
            requestHeaders["Cookie"] = cookieHeader
        }
    }

    if len(requestHeaders) == 0 {
        return nil
    }
    return requestHeaders
}

func BuildCookieHeader(cookies map[string]ParameterSpec) string {
    pairs := make([]string, 0, len(cookies))
    for name, parameter := range cookies {
        if serialized, ok := SerializeParameterValue(parameter); ok {
            pairs = append(pairs, url.QueryEscape(name)+"="+url.QueryEscape(serialized))
        }
    }
    return strings.Join(pairs, "; ")
}

func SerializeParameterValue(parameter ParameterSpec) (string, bool) {
    value := parameter.Value
    if value == nil {
        return "", false
    }
    if parameter.ContentType != "" {
        encoded, _ := json.Marshal(value)
        return string(encoded), true
    }
    switch typed := value.(type) {
    case string:
        return typed, true
    case fmt.Stringer:
        return typed.String(), true
    case []string:
        return strings.Join(typed, ","), true
    case []int:
        values := make([]string, 0, len(typed))
        for _, item := range typed {
            values = append(values, fmt.Sprint(item))
        }
        return strings.Join(values, ","), true
    case map[string]string:
        return SerializeHeaderObject(stringMapToInterface(typed), parameter.Explode), true
    case map[string]int:
        return SerializeHeaderObject(intMapToInterface(typed), parameter.Explode), true
    case map[string]interface{}:
        return SerializeHeaderObject(typed, parameter.Explode), true
    default:
        return fmt.Sprint(value), true
    }
}

func SerializeHeaderObject(values map[string]interface{}, explode bool) string {
    serialized := make([]string, 0, len(values)*2)
    for key, value := range values {
        if value == nil {
            continue
        }
        if explode {
            serialized = append(serialized, key+"="+fmt.Sprint(value))
        } else {
            serialized = append(serialized, key, fmt.Sprint(value))
        }
    }
    return strings.Join(serialized, ",")
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
