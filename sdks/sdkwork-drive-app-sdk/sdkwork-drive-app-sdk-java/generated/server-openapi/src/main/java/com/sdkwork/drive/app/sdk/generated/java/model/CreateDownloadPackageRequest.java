package com.sdkwork.drive.app.sdk.generated.java.model;

import java.util.List;

public class CreateDownloadPackageRequest {
    private List<String> nodeIds;
    private String packageName;
    private Integer requestedTtlSeconds;

    public List<String> getNodeIds() {
        return this.nodeIds;
    }

    public void setNodeIds(List<String> nodeIds) {
        this.nodeIds = nodeIds;
    }

    public String getPackageName() {
        return this.packageName;
    }

    public void setPackageName(String packageName) {
        this.packageName = packageName;
    }

    public Integer getRequestedTtlSeconds() {
        return this.requestedTtlSeconds;
    }

    public void setRequestedTtlSeconds(Integer requestedTtlSeconds) {
        this.requestedTtlSeconds = requestedTtlSeconds;
    }
}
