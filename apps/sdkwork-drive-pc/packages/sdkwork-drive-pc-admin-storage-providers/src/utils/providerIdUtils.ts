import { slugify, uuid } from '@sdkwork/utils';

const MAX_PROVIDER_ID_LENGTH = 64;

export function slugifyProviderIdPart(text: string, maxLen = 20): string {
  const slug = slugify(text).replace(/-/g, '');
  if (slug.length <= maxLen) {
    return slug;
  }
  return slug.slice(0, maxLen);
}

export function createProviderUuidSuffix(): string {
  return uuid();
}

export function buildProviderIdWithUuid(
  kindSlug: string,
  uuidSuffix: string,
  existingIds: ReadonlySet<string> = new Set(),
): string {
  const prefix = kindSlug.trim().slice(0, 20);
  const providerUuid = uuidSuffix.trim();
  if (!prefix || !providerUuid) {
    return '';
  }

  const candidate = `${prefix}-${providerUuid}`.slice(0, MAX_PROVIDER_ID_LENGTH);
  if (!existingIds.has(candidate)) {
    return candidate;
  }

  for (let index = 0; index < 5; index += 1) {
    const retry = buildProviderIdWithUuid(prefix, createProviderUuidSuffix(), existingIds);
    if (retry && !existingIds.has(retry)) {
      return retry;
    }
  }

  return candidate;
}

export function resolveProviderKindSlug(
  providerKind: string,
  customKind: string,
  shortLabel: string,
): string {
  if (providerKind === 'custom') {
    return customKind.trim() ? slugifyProviderIdPart(customKind, 20) : 'custom';
  }
  return slugifyProviderIdPart(shortLabel, 20);
}
