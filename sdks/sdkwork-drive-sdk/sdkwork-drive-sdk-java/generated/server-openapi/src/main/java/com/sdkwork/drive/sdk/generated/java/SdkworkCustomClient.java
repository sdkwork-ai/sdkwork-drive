package com.sdkwork.drive.sdk.generated.java;

import com.sdkwork.common.core.Types;
import com.sdkwork.drive.sdk.generated.java.http.HttpClient;
import com.sdkwork.drive.sdk.generated.java.api.DriveApi;

public class SdkworkCustomClient {
    private final HttpClient httpClient;
    private DriveApi drive;

    public SdkworkCustomClient(String baseUrl) {
        this.httpClient = new HttpClient(baseUrl);
        this.drive = new DriveApi(httpClient);
    }

    public SdkworkCustomClient(Types.SdkConfig config) {
        this.httpClient = new HttpClient(config);
        this.drive = new DriveApi(httpClient);
    }

    public DriveApi getDrive() {
        return this.drive;
    }

    public SdkworkCustomClient setApiKey(String apiKey) {
        httpClient.setApiKey(apiKey);
        return this;
    }

    public SdkworkCustomClient setAuthToken(String token) {
        httpClient.setAuthToken(token);
        return this;
    }

    public SdkworkCustomClient setAccessToken(String token) {
        httpClient.setAccessToken(token);
        return this;
    }

    public SdkworkCustomClient setHeader(String key, String value) {
        httpClient.setHeader(key, value);
        return this;
    }

    public HttpClient getHttpClient() {
        return httpClient;
    }
}
