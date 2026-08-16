package types


type CreateAssetRelationRequest struct {
	RelatedAssetId string `json:"relatedAssetId"`
	RelationType string `json:"relationType"`
	SourceDomain string `json:"sourceDomain"`
	SourceResourceType string `json:"sourceResourceType"`
	SourceResourceId string `json:"sourceResourceId"`
	Metadata map[string]interface{} `json:"metadata"`
}
