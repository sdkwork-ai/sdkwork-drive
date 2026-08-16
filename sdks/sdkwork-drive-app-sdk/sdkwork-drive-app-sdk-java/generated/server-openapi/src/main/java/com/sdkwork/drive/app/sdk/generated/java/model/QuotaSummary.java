package com.sdkwork.drive.app.sdk.generated.java.model;


public class QuotaSummary {
    private String tenantId;
    private Integer usedBytes;
    private Integer objectCount;
    private Integer quotaBytes;

    public String getTenantId() {
        return this.tenantId;
    }

    public void setTenantId(String tenantId) {
        this.tenantId = tenantId;
    }

    public Integer getUsedBytes() {
        return this.usedBytes;
    }

    public void setUsedBytes(Integer usedBytes) {
        this.usedBytes = usedBytes;
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
