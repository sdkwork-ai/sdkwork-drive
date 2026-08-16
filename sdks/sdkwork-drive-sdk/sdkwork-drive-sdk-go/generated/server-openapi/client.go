package generated

type Operation struct {
	Method string
	Path   string
}

const (
	SdkName         = "sdkwork-drive-sdk"
	PackageName     = "sdkwork-drive-sdk-generated-go"
	StandardProfile = "sdkwork-v3"
	BaseURL         = "http://127.0.0.1:18082"
	ApiPrefix       = "/open/v3/api"
)

var Operations = map[string]Operation{
	"openShareLinks.downloadUrls.create": {Method: "POST", Path: "/open/v3/api/drive/share_links/{token}/download_url"},
	"openShareLinks.resolve": {Method: "GET", Path: "/open/v3/api/drive/share_links/{token}"},
}
