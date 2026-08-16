package types


type CreateAssetCollectionRequest struct {
	Title string `json:"title"`
	Description string `json:"description"`
	CollectionType string `json:"collectionType"`
	Visibility string `json:"visibility"`
}
