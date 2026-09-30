import { describe, expect, it } from 'vitest';
import { SPACE_TYPES, getSpaceTypeMeta } from '../src/utils/spaceTypeConfig';

/**
 * The backend's binding-purpose whitelist, mirrored here on purpose.
 *
 * Three copies of the space-type vocabulary exist — `dr_drive_space.space_type`
 * and `dr_drive_node.space_type` in the baseline DDL, the
 * `ck_dr_drive_storage_provider_binding_purpose` check that a space-type binding
 * has to satisfy, and `validate_storage_binding_space_type` on the route — plus
 * this console catalog. A type present in one and missing in another is a defect
 * that shows up late and quietly: `website` sat in the space and node checks (and
 * in the route validator) while the binding check rejected it, so a published-site
 * binding failed at the database instead of working, and the console could not
 * even offer the row. Duplicating the list in a test is what turns that drift
 * into a red test.
 *
 * When adding a space type, add it here too; the assertions below then require
 * the console metadata to catch up.
 */
const BACKEND_SPACE_TYPES = [
  'personal',
  'team',
  'knowledge_base',
  'ai_generated',
  'git_repository',
  'deployment',
  'app_upload',
  'im',
  'rtc',
  'notary',
  'website',
] as const;

describe('space type catalog', () => {
  it('offers every space type the backend accepts for a binding', () => {
    const offered = new Set(SPACE_TYPES.map((meta) => meta.value));
    for (const spaceType of BACKEND_SPACE_TYPES) {
      expect(offered, `bindings table is missing backend space type ${spaceType}`).toContain(
        spaceType,
      );
    }
  });

  it('does not offer a space type the backend would reject', () => {
    for (const meta of SPACE_TYPES) {
      expect(
        BACKEND_SPACE_TYPES as readonly string[],
        `console offers ${meta.value}, which is not a backend space type`,
      ).toContain(meta.value);
    }
  });

  it('carries a label and a description for every space type', () => {
    for (const meta of SPACE_TYPES) {
      expect(meta.labelKey, `${meta.value} has no label key`).not.toBe('');
      expect(meta.descriptionKey, `${meta.value} has no description key`).not.toBe('');
    }
  });

  it('resolves an unknown space type instead of throwing', () => {
    // A space type the console has not caught up with still has to render its
    // row: the binding is real even when the metadata is missing.
    expect(getSpaceTypeMeta('some_future_type')).toMatchObject({
      value: 'some_future_type',
      isSystem: false,
    });
  });
});
