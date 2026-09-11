export declare const sdkMetadata: {
    name: string;
    packageName: string;
    sdkOwner: string;
    apiAuthority: string;
    language: string;
    standardProfile: string;
    baseUrl: string;
    apiPrefix: string;
    sdkDependencies: {
        workspace: string;
        role: string;
        required: boolean;
        dependencyMode: string;
        apiPrefix: string;
        apiAuthority: string;
        generatedTransportImportPolicy: string;
        packageByLanguage: {
            typescript: string;
            rust: string;
            java: string;
            python: string;
            go: string;
        };
    }[];
};
export declare const operations: {
    readonly "storageProviderBindings.default.delete": {
        readonly method: "DELETE";
        readonly path: "/backend/v3/api/drive/storage/bindings/default";
    };
    readonly "storageProviderBindings.default.retrieve": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/bindings/default";
    };
    readonly "storageProviderBindings.default.update": {
        readonly method: "PUT";
        readonly path: "/backend/v3/api/drive/storage/bindings/default";
    };
    readonly "storageProviderBindings.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/bindings";
    };
    readonly "storageProviderKinds.create": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/provider-kinds";
    };
    readonly "storageProviderKinds.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/provider-kinds";
    };
    readonly "storageProviderKinds.update": {
        readonly method: "PATCH";
        readonly path: "/backend/v3/api/drive/storage/provider-kinds/{providerKind}";
    };
    readonly "storageProviders.activate": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/activate";
    };
    readonly "storageProviders.bucket.delete": {
        readonly method: "DELETE";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/bucket";
    };
    readonly "storageProviders.bucket.retrieve": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/bucket";
    };
    readonly "storageProviders.bucket.update": {
        readonly method: "PUT";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/bucket";
    };
    readonly "storageProviders.buckets.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/buckets";
    };
    readonly "storageProviders.capabilities.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/capabilities";
    };
    readonly "storageProviders.create": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers";
    };
    readonly "storageProviders.credentials.rotate": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/credentials/rotate";
    };
    readonly "storageProviders.deactivate": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/deactivate";
    };
    readonly "storageProviders.delete": {
        readonly method: "DELETE";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}";
    };
    readonly "storageProviders.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers";
    };
    readonly "storageProviders.objects.content.retrieve": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/object-contents/{objectKey}";
    };
    readonly "storageProviders.objects.content.update": {
        readonly method: "PUT";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/object-contents/{objectKey}";
    };
    readonly "storageProviders.objects.copy": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/objects/copy";
    };
    readonly "storageProviders.objects.delete": {
        readonly method: "DELETE";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}";
    };
    readonly "storageProviders.objects.list": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/objects";
    };
    readonly "storageProviders.objects.retrieve": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/objects/{objectKey}";
    };
    readonly "storageProviders.retrieve": {
        readonly method: "GET";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}";
    };
    readonly "storageProviders.test": {
        readonly method: "POST";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}/test";
    };
    readonly "storageProviders.update": {
        readonly method: "PATCH";
        readonly path: "/backend/v3/api/drive/storage/providers/{providerId}";
    };
};
//# sourceMappingURL=operations.d.ts.map