import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

function readSource(path: string) {
  return readFileSync(resolve(process.cwd(), path), 'utf8');
}

describe('Drive admin editor layout', () => {
  it('keeps label, binding, and quota editors out of tables and cards', () => {
    const labels = readSource('packages/sdkwork-drive-pc-admin-operations/src/pages/LabelsAdminPage.tsx');
    const bindings = readSource('packages/sdkwork-drive-pc-admin-storage-providers/src/pages/StorageBindingsAdminPage.tsx');
    const quota = readSource('packages/sdkwork-drive-pc-admin-operations/src/pages/QuotaAdminPage.tsx');

    expect(labels).toContain('<Drawer');
    expect(labels).not.toContain('const editing = editingLabelId');
    expect(bindings.match(/<Drawer\b/gu)).toHaveLength(2);
    expect(bindings).not.toContain('const isEditing =');
    expect(quota).toContain('<Drawer');
    expect(quota).toContain('policyEditorOpen');
  });

  it('does not render page-level header regions in admin pages', () => {
    const files = [
      'packages/sdkwork-drive-pc-admin-operations/src/components/OperationsAdminPrimitives.tsx',
      'packages/sdkwork-drive-pc-admin-operations/src/pages/SpacesAdminPage.tsx',
      'packages/sdkwork-drive-pc-admin-storage-providers/src/pages/StorageBindingsAdminPage.tsx',
      'packages/sdkwork-drive-pc-admin-storage-providers/src/pages/StorageProvidersAdminPage.tsx',
    ];

    for (const file of files) {
      const source = readSource(file);
      expect(source).not.toMatch(/<header\b/u);
      expect(source).not.toMatch(/<h1\b/u);
    }
  });
});
