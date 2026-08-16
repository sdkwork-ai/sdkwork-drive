package com.sdkwork.drive.app.sdk.generated.java.model;


public class DownloadPackageItem {
    private String nodeId;
    private String nodeName;
    private String archivePath;
    private String bucket;
    private String objectKey;
    private String contentType;
    private Integer contentLength;
    private String checksumSha256Hex;

    public String getNodeId() {
        return this.nodeId;
    }

    public void setNodeId(String nodeId) {
        this.nodeId = nodeId;
    }

    public String getNodeName() {
        return this.nodeName;
    }

    public void setNodeName(String nodeName) {
        this.nodeName = nodeName;
    }

    public String getArchivePath() {
        return this.archivePath;
    }

    public void setArchivePath(String archivePath) {
        this.archivePath = archivePath;
    }

    public String getBucket() {
        return this.bucket;
    }

    public void setBucket(String bucket) {
        this.bucket = bucket;
    }

    public String getObjectKey() {
        return this.objectKey;
    }

    public void setObjectKey(String objectKey) {
        this.objectKey = objectKey;
    }

    public String getContentType() {
        return this.contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }

    public Integer getContentLength() {
        return this.contentLength;
    }

    public void setContentLength(Integer contentLength) {
        this.contentLength = contentLength;
    }

    public String getChecksumSha256Hex() {
        return this.checksumSha256Hex;
    }

    public void setChecksumSha256Hex(String checksumSha256Hex) {
        this.checksumSha256Hex = checksumSha256Hex;
    }
}
