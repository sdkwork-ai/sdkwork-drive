/**
 * Drive sibling naming — core algorithm lives in @sdkwork/utils.
 * Drive-specific copy semantics stay in this package.
 */

import {
  allocateUniqueSiblingName,
  hasSiblingNameConflict,
  splitDisplayFileName,
} from '@sdkwork/utils';

export {
  allocateUniqueSiblingName,
  hasSiblingNameConflict,
  resolveUniqueSiblingName,
  splitDisplayFileName,
} from '@sdkwork/utils';

const COPY_OF_PREFIX = 'Copy of ';

export function resolveCopyTargetName(
  sourceName: string,
  siblingNames: Iterable<string>,
  sameParent: boolean,
): string {
  const baseName = sameParent ? `${COPY_OF_PREFIX}${sourceName}` : sourceName;
  return allocateUniqueSiblingName(baseName, siblingNames);
}

export function buildCopyOfName(sourceName: string): string {
  return `${COPY_OF_PREFIX}${sourceName}`;
}
