package custom

import (
    "sdkwork-drive-admin-storage-sdk-generated-go/api"
    sdkhttp "sdkwork-drive-admin-storage-sdk-generated-go/http"
)

type SdkworkCustomClient struct {
    http *sdkhttp.Client
    Drive *api.DriveApi
}

func NewSdkworkCustomClient(baseURL string) *SdkworkCustomClient {
    cfg := sdkhttp.NewDefaultConfig(baseURL)
    return NewSdkworkCustomClientWithConfig(cfg)
}

func NewSdkworkCustomClientWithConfig(config sdkhttp.Config) *SdkworkCustomClient {
    client := sdkhttp.NewClient(config)
    return &SdkworkCustomClient{
        http: client,
        Drive: api.NewDriveApi(client),
    }
}

func (c *SdkworkCustomClient) SetApiKey(apiKey string) *SdkworkCustomClient {
    c.http.SetApiKey(apiKey)
    return c
}

func (c *SdkworkCustomClient) SetAuthToken(token string) *SdkworkCustomClient {
    c.http.SetAuthToken(token)
    return c
}

func (c *SdkworkCustomClient) SetAccessToken(token string) *SdkworkCustomClient {
    c.http.SetAccessToken(token)
    return c
}

func (c *SdkworkCustomClient) SetHeader(key string, value string) *SdkworkCustomClient {
    c.http.SetHeader(key, value)
    return c
}

func (c *SdkworkCustomClient) Http() *sdkhttp.Client {
    return c.http
}
