package com.sdkwork.drive.admin.storage.sdk.generated.java.model;

import java.util.List;

public class StorageProviderCapabilities {
    private String providerId;
    private String providerKind;
    private Boolean supportsMultipartUpload;
    private Boolean supportsPresignedUploadPart;
    private Boolean supportsPresignedDownload;
    private Boolean supportsServerSideEncryption;
    private Boolean supportsStorageClass;
    private Boolean supportsCredentialRotation;
    private List<String> supportedServerSideEncryptionModes;
    private List<String> supportedStorageClasses;

    public String getProviderId() {
        return this.providerId;
    }

    public void setProviderId(String providerId) {
        this.providerId = providerId;
    }

    public String getProviderKind() {
        return this.providerKind;
    }

    public void setProviderKind(String providerKind) {
        this.providerKind = providerKind;
    }

    public Boolean getSupportsMultipartUpload() {
        return this.supportsMultipartUpload;
    }

    public void setSupportsMultipartUpload(Boolean supportsMultipartUpload) {
        this.supportsMultipartUpload = supportsMultipartUpload;
    }

    public Boolean getSupportsPresignedUploadPart() {
        return this.supportsPresignedUploadPart;
    }

    public void setSupportsPresignedUploadPart(Boolean supportsPresignedUploadPart) {
        this.supportsPresignedUploadPart = supportsPresignedUploadPart;
    }

    public Boolean getSupportsPresignedDownload() {
        return this.supportsPresignedDownload;
    }

    public void setSupportsPresignedDownload(Boolean supportsPresignedDownload) {
        this.supportsPresignedDownload = supportsPresignedDownload;
    }

    public Boolean getSupportsServerSideEncryption() {
        return this.supportsServerSideEncryption;
    }

    public void setSupportsServerSideEncryption(Boolean supportsServerSideEncryption) {
        this.supportsServerSideEncryption = supportsServerSideEncryption;
    }

    public Boolean getSupportsStorageClass() {
        return this.supportsStorageClass;
    }

    public void setSupportsStorageClass(Boolean supportsStorageClass) {
        this.supportsStorageClass = supportsStorageClass;
    }

    public Boolean getSupportsCredentialRotation() {
        return this.supportsCredentialRotation;
    }

    public void setSupportsCredentialRotation(Boolean supportsCredentialRotation) {
        this.supportsCredentialRotation = supportsCredentialRotation;
    }

    public List<String> getSupportedServerSideEncryptionModes() {
        return this.supportedServerSideEncryptionModes;
    }

    public void setSupportedServerSideEncryptionModes(List<String> supportedServerSideEncryptionModes) {
        this.supportedServerSideEncryptionModes = supportedServerSideEncryptionModes;
    }

    public List<String> getSupportedStorageClasses() {
        return this.supportedStorageClasses;
    }

    public void setSupportedStorageClasses(List<String> supportedStorageClasses) {
        this.supportedStorageClasses = supportedStorageClasses;
    }
}
