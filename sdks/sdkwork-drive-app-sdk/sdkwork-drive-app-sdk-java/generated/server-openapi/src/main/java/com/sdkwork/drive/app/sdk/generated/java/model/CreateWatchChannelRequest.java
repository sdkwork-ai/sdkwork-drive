package com.sdkwork.drive.app.sdk.generated.java.model;


public class CreateWatchChannelRequest {
    private String id;
    private String spaceId;
    private String address;
    private String token;
    private String channelType;
    private Integer expirationEpochMs;

    public String getId() {
        return this.id;
    }

    public void setId(String id) {
        this.id = id;
    }

    public String getSpaceId() {
        return this.spaceId;
    }

    public void setSpaceId(String spaceId) {
        this.spaceId = spaceId;
    }

    public String getAddress() {
        return this.address;
    }

    public void setAddress(String address) {
        this.address = address;
    }

    public String getToken() {
        return this.token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public String getChannelType() {
        return this.channelType;
    }

    public void setChannelType(String channelType) {
        this.channelType = channelType;
    }

    public Integer getExpirationEpochMs() {
        return this.expirationEpochMs;
    }

    public void setExpirationEpochMs(Integer expirationEpochMs) {
        this.expirationEpochMs = expirationEpochMs;
    }
}
