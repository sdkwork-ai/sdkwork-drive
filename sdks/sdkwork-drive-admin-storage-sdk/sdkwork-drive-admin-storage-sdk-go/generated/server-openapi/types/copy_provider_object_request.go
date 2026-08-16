package types


type CopyProviderObjectRequest struct {
	SourceObjectKey string `json:"sourceObjectKey"`
	DestinationObjectKey string `json:"destinationObjectKey"`
	DestinationBucket string `json:"destinationBucket"`
	MetadataDirective string `json:"metadataDirective"`
}
