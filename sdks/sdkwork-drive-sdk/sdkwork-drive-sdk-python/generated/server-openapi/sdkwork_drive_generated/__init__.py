SDK_NAME = "sdkwork-drive-sdk"
PACKAGE_NAME = "sdkwork-drive-sdk-generated-python"
STANDARD_PROFILE = "sdkwork-v3"
BASE_URL = "http://127.0.0.1:18082"
API_PREFIX = "/open/v3/api"

OPERATIONS = {
    "openShareLinks.downloadUrls.create": {"method": "POST", "path": "/open/v3/api/drive/share_links/{token}/download_url"},
    "openShareLinks.resolve": {"method": "GET", "path": "/open/v3/api/drive/share_links/{token}"},
}
