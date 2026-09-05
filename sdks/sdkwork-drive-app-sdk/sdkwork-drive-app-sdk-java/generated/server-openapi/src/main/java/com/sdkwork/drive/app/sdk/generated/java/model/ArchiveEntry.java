package com.sdkwork.drive.app.sdk.generated.java.model;


public class ArchiveEntry {
    private String path;
    private String name;
    private Boolean isDirectory;
    private String uncompressedSizeBytes;
    private String compressedSizeBytes;
    private String contentType;

    public String getPath() {
        return this.path;
    }

    public void setPath(String path) {
        this.path = path;
    }

    public String getName() {
        return this.name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public Boolean getIsDirectory() {
        return this.isDirectory;
    }

    public void setIsDirectory(Boolean isDirectory) {
        this.isDirectory = isDirectory;
    }

    public String getUncompressedSizeBytes() {
        return this.uncompressedSizeBytes;
    }

    public void setUncompressedSizeBytes(String uncompressedSizeBytes) {
        this.uncompressedSizeBytes = uncompressedSizeBytes;
    }

    public String getCompressedSizeBytes() {
        return this.compressedSizeBytes;
    }

    public void setCompressedSizeBytes(String compressedSizeBytes) {
        this.compressedSizeBytes = compressedSizeBytes;
    }

    public String getContentType() {
        return this.contentType;
    }

    public void setContentType(String contentType) {
        this.contentType = contentType;
    }
}
