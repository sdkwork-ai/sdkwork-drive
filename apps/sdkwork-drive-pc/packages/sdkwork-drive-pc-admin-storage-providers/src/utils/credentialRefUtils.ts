export type CredentialInputMode = 'direct' | 'env' | 'secret';

export interface DirectCredentialValues {
  accessKey: string;
  secretKey: string;
  sessionToken?: string;
}

export interface EnvCredentialValues {
  accessKeyEnv: string;
  secretKeyEnv: string;
  sessionTokenEnv?: string;
}

export interface SecretCredentialValues {
  secretRef: string;
}

export interface ParsedCredentialRef {
  mode: CredentialInputMode;
  direct?: DirectCredentialValues;
  env?: EnvCredentialValues;
  secret?: SecretCredentialValues;
}

const MASKED_REF_PATTERN = /^(plain|env|secret|kms|vault):\*{2,3}$/;

export function isCredentialRefMasked(credentialRef?: string): boolean {
  if (!credentialRef) return false;
  return MASKED_REF_PATTERN.test(credentialRef.trim());
}

export function parseCredentialRef(credentialRef: string): ParsedCredentialRef | undefined {
  const trimmed = credentialRef.trim();
  if (!trimmed || isCredentialRefMasked(trimmed)) {
    return undefined;
  }

  if (trimmed.startsWith('plain:')) {
    const parts = trimmed.slice('plain:'.length).split(':');
    if (parts.length < 2) return undefined;
    const [accessKey, secretKey, sessionToken] = parts;
    if (!accessKey || !secretKey) return undefined;
    return {
      mode: 'direct',
      direct: { accessKey, secretKey, sessionToken: sessionToken || undefined },
    };
  }

  if (trimmed.startsWith('env:')) {
    const parts = trimmed.slice('env:'.length).split(':');
    if (parts.length < 2) return undefined;
    const [accessKeyEnv, secretKeyEnv, sessionTokenEnv] = parts;
    if (!accessKeyEnv || !secretKeyEnv) return undefined;
    return {
      mode: 'env',
      env: { accessKeyEnv, secretKeyEnv, sessionTokenEnv: sessionTokenEnv || undefined },
    };
  }

  if (trimmed.startsWith('secret:')) {
    const secretRef = trimmed.slice('secret:'.length).trim();
    if (!secretRef) return undefined;
    return { mode: 'secret', secret: { secretRef } };
  }

  if (trimmed.startsWith('kms:') || trimmed.startsWith('vault:')) {
    const secretRef = trimmed.split(':').slice(1).join(':').trim();
    if (!secretRef) return undefined;
    return { mode: 'secret', secret: { secretRef: trimmed } };
  }

  return undefined;
}

export function buildCredentialRef(
  mode: CredentialInputMode,
  values: DirectCredentialValues | EnvCredentialValues | SecretCredentialValues,
): string | undefined {
  if (mode === 'direct') {
    const { accessKey, secretKey, sessionToken } = values as DirectCredentialValues;
    if (!accessKey.trim() || !secretKey.trim()) return undefined;
    const base = `plain:${accessKey.trim()}:${secretKey.trim()}`;
    return sessionToken?.trim() ? `${base}:${sessionToken.trim()}` : base;
  }

  if (mode === 'env') {
    const { accessKeyEnv, secretKeyEnv, sessionTokenEnv } = values as EnvCredentialValues;
    if (!accessKeyEnv.trim() || !secretKeyEnv.trim()) return undefined;
    const base = `env:${accessKeyEnv.trim()}:${secretKeyEnv.trim()}`;
    return sessionTokenEnv?.trim() ? `${base}:${sessionTokenEnv.trim()}` : base;
  }

  const { secretRef } = values as SecretCredentialValues;
  const ref = secretRef.trim();
  if (!ref) return undefined;
  if (ref.startsWith('secret:') || ref.startsWith('kms:') || ref.startsWith('vault:')) {
    return ref;
  }
  return `secret:${ref}`;
}
