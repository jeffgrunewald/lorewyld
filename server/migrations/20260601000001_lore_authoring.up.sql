-- v1 lore authoring schema: per-server content-author identity, content
-- modules with version chains, settings as worldbuilding workspaces,
-- markdown lore notes with FTS5 search, and a tag taxonomy.
--
-- The pre-existing `users` table (admin auth) is intentionally left
-- alone; `app_user` is the distinct content-authoring identity gated
-- only by the server's join code. Future tiers may unify them.

CREATE TABLE app_user (
    uuid          TEXT     PRIMARY KEY NOT NULL,
    server_uuid   TEXT                 NOT NULL  REFERENCES game_server(id),
    display_name  TEXT                 NOT NULL,
    created_at    TEXT                 NOT NULL  DEFAULT (datetime('now')),
    UNIQUE (server_uuid, display_name)
);

CREATE INDEX idx_app_user_display_name ON app_user(display_name);

CREATE TABLE user_session (
    token       TEXT PRIMARY KEY NOT NULL,
    user_uuid   TEXT             NOT NULL  REFERENCES app_user(uuid) ON DELETE CASCADE,
    created_at  TEXT             NOT NULL  DEFAULT (datetime('now')),
    expires_at  TEXT
);

CREATE INDEX idx_user_session_user_uuid ON user_session(user_uuid);

-- ContentModule: groups records by attribution. Version chain fields
-- (version_string, previous_version_uuid, published_at) support the
-- Promote-to-Module wizard's snapshot-publication semantics.
CREATE TABLE content_module (
    uuid                   TEXT     PRIMARY KEY NOT NULL,
    name                   TEXT                 NOT NULL,
    slug                   TEXT                 NOT NULL  UNIQUE,
    license                TEXT                 NOT NULL,
    license_url            TEXT,
    schema_version         INTEGER              NOT NULL  DEFAULT 1,
    release_date           TEXT,
    authors                TEXT                 NOT NULL  DEFAULT '[]',
    publisher              TEXT,
    description            TEXT,
    website_url            TEXT,
    is_active              INTEGER              NOT NULL  DEFAULT 1,
    ordering               INTEGER              NOT NULL  DEFAULT 0,
    version_string         TEXT                 NOT NULL  DEFAULT '1.0.0',
    previous_version_uuid  TEXT                                       REFERENCES content_module(uuid),
    published_at           TEXT,
    created_at             TEXT                 NOT NULL  DEFAULT (datetime('now')),
    updated_at             TEXT                 NOT NULL  DEFAULT (datetime('now'))
);

CREATE INDEX idx_content_module_slug ON content_module(slug);
CREATE INDEX idx_content_module_version_chain ON content_module(previous_version_uuid);

CREATE TABLE setting (
    uuid                       TEXT     PRIMARY KEY NOT NULL,
    name                       TEXT                 NOT NULL,
    description_note_uuid      TEXT,
    owner_user_uuid            TEXT                 NOT NULL  REFERENCES app_user(uuid),
    published_as_module_uuid   TEXT                                       REFERENCES content_module(uuid),
    created_at                 TEXT                 NOT NULL  DEFAULT (datetime('now')),
    updated_at                 TEXT                 NOT NULL  DEFAULT (datetime('now'))
);

CREATE INDEX idx_setting_owner ON setting(owner_user_uuid);

CREATE TABLE setting_collaborator (
    setting_uuid  TEXT NOT NULL REFERENCES setting(uuid) ON DELETE CASCADE,
    user_uuid     TEXT NOT NULL REFERENCES app_user(uuid),
    PRIMARY KEY (setting_uuid, user_uuid)
);

-- LoreNote: unstructured markdown content. The scope_kind discriminator
-- selects which entity scope_target_uuid points at (module / setting /
-- campaign / character). The Module/Setting variants are wired in v1;
-- Campaign/Character arrive in v1.5 when their respective tables land.
CREATE TABLE lore_note (
    uuid                              TEXT     PRIMARY KEY NOT NULL,
    title                             TEXT                 NOT NULL,
    body_markdown                     TEXT                 NOT NULL  DEFAULT '',
    scope_kind                        TEXT                 NOT NULL,  -- 'module' | 'setting' | 'campaign' | 'character'
    scope_target_uuid                 TEXT                 NOT NULL,
    visibility                        TEXT                 NOT NULL  DEFAULT 'visible',  -- 'visible' | 'author_only' | 'gamemaster_only'
    derived_from_setting_note_uuid    TEXT                                       REFERENCES lore_note(uuid),
    created_by_user_uuid              TEXT                 NOT NULL  REFERENCES app_user(uuid),
    created_at                        TEXT                 NOT NULL  DEFAULT (datetime('now')),
    updated_at                        TEXT                 NOT NULL  DEFAULT (datetime('now'))
);

CREATE INDEX idx_lore_note_scope ON lore_note(scope_kind, scope_target_uuid);
CREATE INDEX idx_lore_note_author ON lore_note(created_by_user_uuid);
CREATE INDEX idx_lore_note_derived_from ON lore_note(derived_from_setting_note_uuid);

-- FTS5 virtual table mirroring title + body_markdown for fast text
-- search. Kept in sync via triggers on the base lore_note table.
CREATE VIRTUAL TABLE lore_note_fts USING fts5(
    title,
    body_markdown,
    content='lore_note',
    content_rowid='rowid'
);

-- External-content FTS5 requires the 'delete' command with OLD column
-- values to properly purge the internal index — a plain DELETE / UPDATE
-- on the virtual table leaves stale tokens behind.
-- https://www.sqlite.org/fts5.html#external_content_tables
CREATE TRIGGER lore_note_fts_insert AFTER INSERT ON lore_note BEGIN
    INSERT INTO lore_note_fts(rowid, title, body_markdown)
    VALUES (new.rowid, new.title, new.body_markdown);
END;

CREATE TRIGGER lore_note_fts_update AFTER UPDATE ON lore_note BEGIN
    INSERT INTO lore_note_fts(lore_note_fts, rowid, title, body_markdown)
    VALUES ('delete', old.rowid, old.title, old.body_markdown);
    INSERT INTO lore_note_fts(rowid, title, body_markdown)
    VALUES (new.rowid, new.title, new.body_markdown);
END;

CREATE TRIGGER lore_note_fts_delete AFTER DELETE ON lore_note BEGIN
    INSERT INTO lore_note_fts(lore_note_fts, rowid, title, body_markdown)
    VALUES ('delete', old.rowid, old.title, old.body_markdown);
END;

-- Tag taxonomy. Slugs are globally unique within the server; user-tag
-- collisions merge by default (no per-module namespacing enforcement).
CREATE TABLE tag (
    uuid                       TEXT     PRIMARY KEY NOT NULL,
    slug                       TEXT                 NOT NULL  UNIQUE,
    display_name               TEXT                 NOT NULL,
    is_system                  INTEGER              NOT NULL  DEFAULT 0,
    introduced_by_module_uuid  TEXT                                       REFERENCES content_module(uuid),
    created_at                 TEXT                 NOT NULL  DEFAULT (datetime('now'))
);

CREATE TABLE tag_alias (
    tag_uuid    TEXT NOT NULL REFERENCES tag(uuid) ON DELETE CASCADE,
    alias_slug  TEXT NOT NULL UNIQUE,
    PRIMARY KEY (tag_uuid, alias_slug)
);

-- One attachment table per attachable record type. SQLite query planner
-- handles per-type joins much better than a single polymorphic table.
CREATE TABLE tag_attachment_lore_note (
    tag_uuid        TEXT NOT NULL REFERENCES tag(uuid) ON DELETE CASCADE,
    lore_note_uuid  TEXT NOT NULL REFERENCES lore_note(uuid) ON DELETE CASCADE,
    PRIMARY KEY (tag_uuid, lore_note_uuid)
);

CREATE INDEX idx_tag_attachment_lore_note_note ON tag_attachment_lore_note(lore_note_uuid);

-- Seed well-known system tags. UUIDs generated via SQLite's randomblob.
INSERT INTO tag (uuid, slug, display_name, is_system) VALUES
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'lore',           'Lore',           1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'worldbuilding',  'Worldbuilding',  1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'npc',            'NPC',            1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'location',       'Location',       1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'faction',        'Faction',        1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'event',          'Event',          1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'session-recap',  'Session Recap',  1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'backstory',      'Backstory',      1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'plot-hook',      'Plot Hook',      1),
    (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-4' || substr(lower(hex(randomblob(2))), 2) || '-' || substr('89ab', 1 + (abs(random()) % 4), 1) || substr(lower(hex(randomblob(2))), 2) || '-' || lower(hex(randomblob(6))), 'treasure',       'Treasure',       1);
