package com.sdkwork.drive.admin.storage.sdk.generated.java.model;


public class CreateStorageProviderRequest {
    private String id;
    private String providerKind;
    private String name;
    private String endpointUrl;
    private String region;
    private String bucket;
    private Boolean pathStyle;
    private String credentialRef;
    private String serverSideEncryptionMode;
    private String defaultStorageClass;
    private String status;
    private Boolean strictTls;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getProviderKind() {
        return this.providerKind;
    }

    public void setProviderKind(String providerKind) {
        this.providerKind = providerKind;
    }

    public String getName() {
        return this.name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getEndpointUrl() {
        return this.endpointUrl;
    }

    public void setEndpointUrl(String endpointUrl) {
        this.endpointUrl = endpointUrl;
    }

    public String getRegion() {
        return this.region;
    }

    public void setRegion(String region) {
        this.region = region;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public Boolean getPathStyle() {
        return this.pathStyle;
    }

    public void setPathStyle(Boolean pathStyle) {
        this.pathStyle = pathStyle;
    }

    public String getCredentialRef() {
        return this.credentialRef;
    }

    public void setCredentialRef(String credentialRef) {
        this.credentialRef = credentialRef;
    }

    public String getServerSideEncryptionMode() {
        return this.serverSideEncryptionMode;
    }

    public void setServerSideEncryptionMode(String serverSideEncryptionMode) {
        this.serverSideEncryptionMode = serverSideEncryptionMode;
    }

    public String getDefaultStorageClass() {
        return this.defaultStorageClass;
    }

    public void setDefaultStorageClass(String defaultStorageClass) {
        this.defaultStorageClass = defaultStorageClass;
    }

    public String getStatus() {
        return this.status;
    }

    public void setStatus(String status) {
        this.status = status;
    }

    public Boolean getStrictTls() {
        return this.strictTls;
    }

    public void setStrictTls(Boolean strictTls) {
        this.strictTls = strictTls;
    }
}
