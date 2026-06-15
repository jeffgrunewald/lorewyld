-- D&D SRD content tables, aligned with the Open5e-v2-shaped bundle
-- schema (shared/types SCHEMA_VERSION 2).
--
-- Doc-style hybrid layout: every table carries identity + a few indexed
-- filter columns, with the full record JSON in `data`. Content is
-- read-only reference data imported from ContentBundle files (see
-- server/src/content.rs), so partial column updates never happen and
-- serialization stays zero-mapping through lorewyld-types.
--
-- Display content tables additionally carry a `summary` column: the
-- list-row projection serialized by each type's `summary()` (e.g.
-- `Spell::summary` -> `SpellSummary`). It is single-sourced in
-- lorewyld-types and served verbatim by the compendium list endpoint, so
-- the slim list shape is never assembled in SQL. Lookup tables return
-- full records and have no summary column.
--
-- FTS5 content search is deferred; when it lands, follow the
-- lore_note external-content pattern from the lore_authoring migration.

-- Provenance ------------------------------------------------------------

CREATE TABLE license (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE publisher (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE document (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

-- Lookup tables ----------------------------------------------------------

CREATE TABLE ability_score (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE skill (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE alignment (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE damage_type (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE condition (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE language (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE size (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE environment (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE spell_school (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE creature_type (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE item_category (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE weapon_property (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    data                 TEXT NOT NULL
);

-- Major content ----------------------------------------------------------
-- Display tables carry a single-sourced `summary` column (see header).

CREATE TABLE spell (
    uuid                 TEXT    PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT    NOT NULL REFERENCES content_module(uuid),
    key                  TEXT    NOT NULL UNIQUE,
    slug                 TEXT    NOT NULL,
    name                 TEXT    NOT NULL,
    level                INTEGER NOT NULL,
    school_uuid          TEXT    NOT NULL REFERENCES spell_school(uuid),
    concentration        INTEGER NOT NULL DEFAULT 0,
    ritual               INTEGER NOT NULL DEFAULT 0,
    summary              TEXT    NOT NULL,
    data                 TEXT    NOT NULL
);

CREATE INDEX idx_spell_level ON spell(level);
CREATE INDEX idx_spell_school ON spell(school_uuid);

CREATE TABLE creature (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    challenge_rating     REAL NOT NULL,
    creature_type_uuid   TEXT NOT NULL REFERENCES creature_type(uuid),
    size_uuid            TEXT NOT NULL REFERENCES size(uuid),
    summary              TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE INDEX idx_creature_cr ON creature(challenge_rating);
CREATE INDEX idx_creature_type ON creature(creature_type_uuid);

CREATE TABLE class (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    subclass_of          TEXT REFERENCES class(uuid),
    summary              TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE INDEX idx_class_parent ON class(subclass_of);

CREATE TABLE species (
    uuid                 TEXT    PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT    NOT NULL REFERENCES content_module(uuid),
    key                  TEXT    NOT NULL UNIQUE,
    slug                 TEXT    NOT NULL,
    name                 TEXT    NOT NULL,
    is_subspecies        INTEGER NOT NULL DEFAULT 0,
    summary              TEXT    NOT NULL,
    data                 TEXT    NOT NULL
);

CREATE TABLE feat (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    summary              TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE background (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    summary              TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE weapon (
    uuid                 TEXT    PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT    NOT NULL REFERENCES content_module(uuid),
    key                  TEXT    NOT NULL UNIQUE,
    slug                 TEXT    NOT NULL,
    name                 TEXT    NOT NULL,
    is_simple            INTEGER NOT NULL DEFAULT 0,
    summary              TEXT    NOT NULL,
    data                 TEXT    NOT NULL
);

CREATE TABLE armor (
    uuid                 TEXT PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT NOT NULL REFERENCES content_module(uuid),
    key                  TEXT NOT NULL UNIQUE,
    slug                 TEXT NOT NULL,
    name                 TEXT NOT NULL,
    category             TEXT NOT NULL,
    summary              TEXT NOT NULL,
    data                 TEXT NOT NULL
);

CREATE TABLE item (
    uuid                 TEXT    PRIMARY KEY NOT NULL,
    content_module_uuid  TEXT    NOT NULL REFERENCES content_module(uuid),
    key                  TEXT    NOT NULL UNIQUE,
    slug                 TEXT    NOT NULL,
    name                 TEXT    NOT NULL,
    category_uuid        TEXT    NOT NULL REFERENCES item_category(uuid),
    rarity               TEXT,
    is_magic             INTEGER NOT NULL DEFAULT 0,
    summary              TEXT    NOT NULL,
    data                 TEXT    NOT NULL
);

CREATE INDEX idx_item_category ON item(category_uuid);
CREATE INDEX idx_item_magic ON item(is_magic);
