-- Inc 3a: materialized list-summary column for spell.
--
-- Populated in Rust from `Spell::summary()` at ingest, and backfilled at
-- boot for rows seeded before this column existed (see
-- server/src/content.rs::backfill_summaries). The compendium list endpoint
-- serves this column verbatim, so the list-row shape is single-sourced in
-- lorewyld-types (SpellSummary) rather than assembled in SQL.
ALTER TABLE spell ADD COLUMN summary TEXT;
