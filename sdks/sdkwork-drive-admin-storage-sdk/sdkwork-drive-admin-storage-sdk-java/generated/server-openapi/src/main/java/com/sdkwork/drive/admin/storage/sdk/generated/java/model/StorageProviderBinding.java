package com.sdkwork.drive.admin.storage.sdk.generated.java.model;


public class StorageProviderBinding {
    private String id;
    private String tenantId;
    private String spaceId;
    private String providerId;
    private String bindingScope;
    private String purpose;
    private String lifecycleStatus;
    private Integer version;
    private StorageProvider storageProvider;
    private String storageRootPrefix;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getTenantId() {
        return this.tenantId;
    }

    public void setTenantId(String tenantId) {
        this.tenantId = tenantId;
    }

    public String getSpaceId() {
        return this.spaceId;
    }

    public void setSpaceId(String spaceId) {
        this.spaceId = spaceId;
    }

    public String getProviderId() {
        return this.providerId;
    }

    public void setProviderId(String providerId) {
        this.providerId = providerId;
    }

    public String getBindingScope() {
        return this.bindingScope;
    }

    public void setBindingScope(String bindingScope) {
        this.bindingScope = bindingScope;
    }

    public String getPurpose() {
        return this.purpose;
    }

    public void setPurpose(String purpose) {
        this.purpose = purpose;
    }

    public String getLifecycleStatus() {
        return this.lifecycleStatus;
    }

    public void setLifecycleStatus(String lifecycleStatus) {
        this.lifecycleStatus = lifecycleStatus;
    }

    public Integer getVersion() {
        return this.version;
    }

    public void setVersion(Integer version) {
        this.version = version;
    }

    public StorageProvider getStorageProvider() {
        return this.storageProvider;
    }

    public void setStorageProvider(StorageProvider storageProvider) {
        this.storageProvider = storageProvider;
    }

    public String getStorageRootPrefix() {
        return this.storageRootPrefix;
    }

    public void setStorageRootPrefix(String storageRootPrefix) {
        this.storageRootPrefix = storageRootPrefix;
    }
}
