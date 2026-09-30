-- sdkwork:migration
-- id: 0006_drive_storage_provider_kind_translation
-- engine: postgres
-- module: sdkwork-drive
-- purpose: Localized display names for the built-in storage provider-kind
--   catalog (I18N_SPEC.md §11, DATABASE_SPEC.md §6.4.1). The base table keeps
--   the stable machine fields plus the locale-neutral fallback display name;
--   this translation table carries the per-locale operator-facing label keyed
--   by (provider_kind, locale). Translation rows are installed by the
--   idempotent locale seed scripts under database/seeds/locales/<locale>/, so
--   adding or activating a locale never needs another schema migration.
-- reversible: true
-- rollback: down-migration
-- transactional: true
-- lock: lightweight
-- lock_timeout: 2s
-- statement_timeout: 30s

BEGIN;

CREATE TABLE IF NOT EXISTS dr_drive_storage_provider_kind_translation (
    provider_kind VARCHAR(64) NOT NULL,
    locale VARCHAR(16) NOT NULL,
    display_name VARCHAR(128) NOT NULL,
    version BIGINT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT pk_dr_drive_storage_provider_kind_translation
        PRIMARY KEY (provider_kind, locale),
    CONSTRAINT fk_dr_drive_storage_provider_kind_translation_kind
        FOREIGN KEY (provider_kind)
        REFERENCES dr_drive_storage_provider_kind (provider_kind)
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_dr_drive_storage_provider_kind_translation_locale
    ON dr_drive_storage_provider_kind_translation (locale);

COMMIT;
