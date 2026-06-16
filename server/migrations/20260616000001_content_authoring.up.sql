-- Homebrew content authoring: per-record authorship.
--
-- User-authored content records (origin = 'local' modules) carry the
-- uuid of the user who created them, so edit/delete can be gated to the
-- creator or an admin — mirroring lore_note.created_by_user_uuid.
-- Imported/seeded reference records leave this NULL.
--
-- Only the user-authorable display categories gain the column; the
-- shared-vocabulary lookup tables (size, spell_school, …) are never
-- user-authored, except condition and language which ARE display
-- categories users may author.

ALTER TABLE spell      ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE creature   ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE class      ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE species    ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE background ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE feat       ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE item       ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE weapon     ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE armor      ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE condition  ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
ALTER TABLE language   ADD COLUMN created_by_user_uuid TEXT REFERENCES users(id);
