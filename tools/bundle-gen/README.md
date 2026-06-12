# bundle-gen

Generates the shipped SRD content bundle (`content/srd-bundle.json` plus the
byte-identical Flutter asset copy at `mobile/assets/content/srd-bundle.json`)
from the [Open5e API](https://api.open5e.com).

## What it builds

- **SRD 5.2** (`srd-2024`) is the base; **SRD 5.1** (`srd-2014`) records whose
  normalized name is missing from 5.2 are gap-filled in. Known renames
  (Feeblemind→Befuddlement, "Crossbow, light"→"Light Crossbow", …) dedup via
  the alias map in `data/overrides.json`.
- **Sheet-math recovery**: Open5e v2 dropped class proficiencies/equipment/
  spell-slot data and species ASI/speed/size to prose. The generator joins the
  Open5e **v1** API (classes, races) and the curated tables in
  `data/overrides.json` (canonical spell-slot tables, 2024 species-ASI policy)
  to keep those fields structured.
- **Deterministic output**: UUIDv5 identities derived from Open5e keys, pinned
  timestamps, records sorted by key — re-running against unchanged upstream
  data produces a zero git diff.

## Usage

```sh
cargo run -p bundle-gen --release
```

Raw API pages are cached in `tools/bundle-gen/.cache/` (gitignored); delete it
to refetch from upstream. The run prints a per-table count summary and the
full list of gap-filled 5.1 records for review.

## Upstream quirks the generator works around

- `document__key` filtering is broken on the `items`, `magicitems`, and
  `weapons` endpoints (returns the unfiltered set) — everything is re-filtered
  client-side on the embedded `document.key`.
- srd-2014 structured creature attacks carry a corrupt `damage_type`
  (almost everything claims "thunder") and omit `damage_bonus`; the typed
  damage fields are dropped for that edition. The prose `desc` on each action
  is authoritative and complete.
- Several numeric fields occasionally arrive as prose (`"30 feet"`), and
  `null` appears where the schema implies a scalar — the deserializers in
  `src/v2.rs` are tolerant by design.
