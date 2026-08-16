package com.sdkwork.drive.backend.sdk.generated.java.model;


public class QuotaSummary {
    private String tenantId;
    private Integer totalBytes;
    private Integer objectCount;
    private Integer quotaBytes;

    public String getTenantId() {
        return this.tenantId;
    }

    public void setTenantId(String tenantId) {
        this.tenantId = tenantId;
    }

    public Integer getTotalBytes() {
        return this.totalBytes;
    }

    public void setTotalBytes(Integer totalBytes) {
        this.totalBytes = totalBytes;
    }

    public Integer getObjectCount() {
        return this.objectCount;
    }

    public void setObjectCount(Integer objectCount) {
        this.objectCount = objectCount;
    }

    public Integer getQuotaBytes() {
        return this.quotaBytes;
    }

    public void setQuotaBytes(Integer quotaBytes) {
        this.quotaBytes = quotaBytes;
    }
}
