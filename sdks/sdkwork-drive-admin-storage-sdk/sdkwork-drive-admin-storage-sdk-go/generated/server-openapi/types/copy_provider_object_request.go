package types


type CopyProviderObjectRequest struct {
	SourceObjectKey string `json:"sourceObjectKey"`
	DestinationObjectKey string `json:"destinationObjectKey"`
	SourceBucket string `json:"sourceBucket"`
	DestinationBucket string `json:"destinationBucket"`
	Region string `json:"region"`
	MetadataDirective string `json:"metadataDirective"`
}
