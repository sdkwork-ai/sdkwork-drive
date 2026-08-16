import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

const componentRoot = resolve(process.cwd(), 'packages/sdkwork-drive-pc-admin-storage-providers/src/components');

function readComponent(name: string) {
  return readFileSync(resolve(componentRoot, name), 'utf8');
}

describe('storage provider editor layout', () => {
  it('uses the shared operation drawer for create, edit, and detail workflows', () => {
    const editor = readComponent('StorageProviderEditor.tsx');
    const detail = readComponent('StorageProviderDetailDrawer.tsx');

    for (const source of [editor, detail]) {
      expect(source).toContain('<OperationDrawer');
      expect(source).not.toContain('className="fixed inset-0');
      expect(source).not.toContain('role="dialog"');
      expect(source).not.toContain('aria-modal="true"');
    }
  });

  it('keeps detail tabs accessible and uses framework icons', () => {
    const detail = readComponent('StorageProviderDetailDrawer.tsx');

    expect(detail).toContain('role="tablist"');
    expect(detail).toContain('aria-selected={tab ===');
    expect(detail).not.toContain('<svg');
  });
});
