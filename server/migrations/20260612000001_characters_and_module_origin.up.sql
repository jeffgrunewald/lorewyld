-- Module provenance + server-side character sheets.
--
-- `origin` distinguishes how a content_module landed on this server:
--   'bundled'   — seeded from the embedded content bundle at boot; can
--                 only be disabled (the seeder would re-add a deleted
--                 bundled module on next boot), never uninstalled.
--   'uploaded'  — installed by an admin from a ContentBundle package;
--                 fully uninstallable.
--   'published' — created via the Promote-to-Module wizard from a
--                 Setting's lore notes; fully uninstallable.
--
-- The migration cannot know the embedded bundle's slugs, so existing
-- bundled rows are corrected by the seeder's idempotent origin stamp on
-- the next boot; 'published' rows are recoverable from the setting link.

ALTER TABLE content_module ADD COLUMN origin TEXT NOT NULL DEFAULT 'uploaded'
    CHECK (origin IN ('bundled', 'uploaded', 'published'));

UPDATE content_module SET origin = 'published'
 WHERE uuid IN (SELECT published_as_module_uuid FROM setting
                 WHERE published_as_module_uuid IS NOT NULL);

-- Server-side 5e character sheets, owned per-user. Doc-style hybrid
-- like the content tables: identity columns + full sheet JSON in data.
CREATE TABLE character (
    uuid             TEXT PRIMARY KEY NOT NULL,
    owner_user_uuid  TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    data             TEXT NOT NULL,
    created_at       TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at       TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_character_owner ON character(owner_user_uuid);
