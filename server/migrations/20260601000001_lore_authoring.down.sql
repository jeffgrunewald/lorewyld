DROP TRIGGER IF EXISTS lore_note_fts_delete;
DROP TRIGGER IF EXISTS lore_note_fts_update;
DROP TRIGGER IF EXISTS lore_note_fts_insert;
DROP TABLE IF EXISTS lore_note_fts;

DROP INDEX IF EXISTS idx_tag_attachment_lore_note_note;
DROP TABLE IF EXISTS tag_attachment_lore_note;
DROP TABLE IF EXISTS tag_alias;
DROP TABLE IF EXISTS tag;

DROP INDEX IF EXISTS idx_lore_note_derived_from;
DROP INDEX IF EXISTS idx_lore_note_author;
DROP INDEX IF EXISTS idx_lore_note_scope;
DROP TABLE IF EXISTS lore_note;

DROP TABLE IF EXISTS setting_collaborator;
DROP INDEX IF EXISTS idx_setting_owner;
DROP TABLE IF EXISTS setting;

DROP INDEX IF EXISTS idx_content_module_version_chain;
DROP TABLE IF EXISTS content_module;

DROP INDEX IF EXISTS idx_user_session_user_uuid;
DROP TABLE IF EXISTS user_session;

DROP INDEX IF EXISTS idx_app_user_display_name;
DROP TABLE IF EXISTS app_user;
