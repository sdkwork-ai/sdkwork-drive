package com.sdkwork.drive.app.sdk.generated.java;

import com.sdkwork.common.core.Types;
import com.sdkwork.drive.app.sdk.generated.java.http.HttpClient;
import com.sdkwork.drive.app.sdk.generated.java.api.DriveApi;
import com.sdkwork.drive.app.sdk.generated.java.api.AssetsApi;

public class SdkworkAppClient {
    private final HttpClient httpClient;
    private DriveApi drive;
    private AssetsApi assets;

    public SdkworkAppClient(String baseUrl) {
        this.httpClient = new HttpClient(baseUrl);
        this.drive = new DriveApi(httpClient);
        this.assets = new AssetsApi(httpClient);
    }

    public SdkworkAppClient(Types.SdkConfig config) {
        this.httpClient = new HttpClient(config);
        this.drive = new DriveApi(httpClient);
        this.assets = new AssetsApi(httpClient);
    }

    public DriveApi getDrive() {
        return this.drive;
    }

    public AssetsApi getAssets() {
        return this.assets;
    }
    public SdkworkAppClient setAuthToken(String token) {
        httpClient.setAuthToken(token);
        return this;
    }

    public SdkworkAppClient setAccessToken(String token) {
        httpClient.setAccessToken(token);
        return this;
    }

    public SdkworkAppClient setHeader(String key, String value) {
        httpClient.setHeader(key, value);
        return this;
    }

    public HttpClient getHttpClient() {
        return httpClient;
    }
}
