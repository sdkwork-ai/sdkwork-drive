package com.sdkwork.generated;

import java.util.LinkedHashMap;
import java.util.Map;

public final class SdkMetadata {
  public static final String SDK_NAME = "sdkwork-drive-sdk";
  public static final String PACKAGE_NAME = "sdkwork-drive-sdk-generated-java";
  public static final String STANDARD_PROFILE = "sdkwork-v3";
  public static final String BASE_URL = "http://127.0.0.1:18082";
  public static final String API_PREFIX = "/open/v3/api";

  public static Map<String, String> operations() {
    Map<String, String> operations = new LinkedHashMap<>();
    operations.put("openShareLinks.downloadUrls.create", "POST /open/v3/api/drive/share_links/{token}/download_url");
    operations.put("openShareLinks.resolve", "GET /open/v3/api/drive/share_links/{token}");
    return operations;
  }

  private SdkMetadata() {}
}
