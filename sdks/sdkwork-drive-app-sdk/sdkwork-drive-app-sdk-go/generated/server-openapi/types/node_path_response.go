package types


type NodePathResponse struct {
	Items []DriveNode `json:"items"`
	PathSegments []string `json:"pathSegments"`
}
