import React from 'react';
import type {
  StorageProviderBucketView,
  StorageProviderCapabilitiesView,
  StorageProviderView,
} from '../types/storageProviderAdminTypes';
import { getProviderKindMeta, HEALTH_STATUS_CONFIG } from '../utils/providerKindConfig';
import { SECONDARY_BUTTON_CLASS, CARD_CLASS } from '../utils/uiPrimitives';

interface StorageProviderDiagnosticsPanelProps {
  provider?: StorageProviderView;
  capabilities?: StorageProviderCapabilitiesView;
  bucket?: StorageProviderBucketView;
  onLoadCapabilities: (providerId: string) => void;
  onHeadBucket: (providerId: string) => void;
  pending?: boolean;
}

export function StorageProviderDiagnosticsPanel({
  provider,
  capabilities,
  bucket,
  onLoadCapabilities,
  onHeadBucket,
  pending,
}: StorageProviderDiagnosticsPanelProps) {
  if (!provider) {
    return (
      <div className={CARD_CLASS}>
        <div className="px-5 py-6 text-center text-sm text-neutral-500 dark:text-neutral-400">
          Select a provider to view diagnostics.
        </div>
      </div>
    );
  }

  const meta = getProviderKindMeta(provider.providerKind);

  return (
    <div className={CARD_CLASS}>
      <div className="border-b border-neutral-100 px-5 py-3 dark:border-neutral-800">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={`flex h-7 w-7 items-center justify-center rounded text-xs font-bold ${meta.bgClass} ${meta.textClass}`}>
              {meta.icon}
            </div>
            <div>
              <h3 className="text-sm font-semibold text-neutral-900 dark:text-neutral-100">Diagnostics</h3>
              <p className="text-[11px] text-neutral-500 dark:text-neutral-400">
                Connectivity and capability checks
              </p>
            </div>
          </div>
          <div className="flex gap-2">
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={pending}
              onClick={() => onLoadCapabilities(provider.id)}
            >
              <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Capabilities
            </button>
            <button
              type="button"
              className={SECONDARY_BUTTON_CLASS}
              disabled={pending}
              onClick={() => onHeadBucket(provider.id)}
            >
              <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
              </svg>
              Bucket check
            </button>
          </div>
        </div>
      </div>

      <div className="px-5 py-4">
        {/* Health status */}
        <div className="mb-4 flex items-center gap-3 rounded-md border border-neutral-100 p-3 dark:border-neutral-800">
          <div className={`flex h-8 w-8 items-center justify-center rounded-full ${
            HEALTH_STATUS_CONFIG[provider.healthStatus ?? 'unknown'].bgClass
          }`}>
            <span className={`h-3 w-3 rounded-full ${HEALTH_STATUS_CONFIG[provider.healthStatus ?? 'unknown'].dotClass}`} />
          </div>
          <div>
            <div className="text-sm font-medium text-neutral-900 dark:text-neutral-100">
              Provider health: {HEALTH_STATUS_CONFIG[provider.healthStatus ?? 'unknown'].label}
            </div>
            <div className="font-mono text-[11px] leading-relaxed break-all text-neutral-500 dark:text-neutral-400">
              {provider.endpointUrl} · {provider.bucket}
            </div>
          </div>
        </div>

        {/* Bucket reachability */}
        {bucket && (
          <div className="mb-4">
            <div className="text-xs font-semibold text-neutral-500 dark:text-neutral-400 mb-2">Bucket reachability</div>
            <div className={`flex items-center gap-2 rounded-md p-2 text-xs ${
              bucket.exists
                ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/30 dark:text-emerald-300'
                : 'bg-red-50 text-red-700 dark:bg-red-950/30 dark:text-red-300'
            }`}>
              {bucket.exists ? (
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              ) : (
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              )}
              {bucket.exists ? `Bucket "${bucket.bucket}" is reachable` : `Bucket "${bucket.bucket}" does not exist or is unreachable`}
            </div>
          </div>
        )}

        {/* Capabilities grid */}
        {capabilities && (
          <div>
            <div className="text-xs font-semibold text-neutral-500 dark:text-neutral-400 mb-2">Capabilities</div>
            <div className="grid grid-cols-2 gap-2">
              <CapabilityCard
                label="Multipart upload"
                supported={capabilities.supportsMultipartUpload}
                description="Large file upload support"
              />
              <CapabilityCard
                label="Presigned upload"
                supported={capabilities.supportsPresignedUploadPart}
                description="Direct client upload"
              />
              <CapabilityCard
                label="Presigned download"
                supported={capabilities.supportsPresignedDownload}
                description="Direct client download"
              />
              <CapabilityCard
                label="Server-side encryption"
                supported={capabilities.supportsServerSideEncryption}
                description={capabilities.supportedServerSideEncryptionModes.join(', ') || 'Not available'}
              />
              <CapabilityCard
                label="Storage classes"
                supported={capabilities.supportsStorageClass}
                description={capabilities.supportedStorageClasses.slice(0, 3).join(', ') || 'Not available'}
              />
              <CapabilityCard
                label="Credential rotation"
                supported={capabilities.supportsCredentialRotation}
                description="Live credential update"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function CapabilityCard({ label, supported, description }: { label: string; supported: boolean; description: string }) {
  return (
    <div className={`rounded-md border p-2.5 ${
      supported
        ? 'border-emerald-200 bg-emerald-50/50 dark:border-emerald-900/50 dark:bg-emerald-950/20'
        : 'border-neutral-200 bg-neutral-50 dark:border-neutral-800 dark:bg-neutral-900/50'
    }`}>
      <div className="flex items-center gap-1.5">
        {supported ? (
          <svg className="h-3.5 w-3.5 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
          </svg>
        ) : (
          <svg className="h-3.5 w-3.5 text-neutral-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        )}
        <span className="text-xs font-medium text-neutral-900 dark:text-neutral-100">{label}</span>
      </div>
      <p className="mt-1 text-[10px] text-neutral-500 dark:text-neutral-400 leading-relaxed">{description}</p>
    </div>
  );
}
