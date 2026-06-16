# Architecture

## System Overview

Lorewyld is a self-hostable D&D content platform. A single Rust binary serves
both the JSON REST API and a server-rendered web client; a companion Flutter app
provides a local-first mobile client. All three surfaces — server, web, and
mobile — share one pure-Rust core so the 5e rules math and schema cannot drift
between platforms.

```
            ┌───────────────────────────── Shared Rust core ─────────────────────────────┐
            │  lorewyld-types (schema)   ·   lorewyld-domain (5e rules math / validation)  │
            └───────┬──────────────────────────┬───────────────────────────────┬──────────┘
            native  │                  WASM     │                       FFI     │
                    ▼                           ▼                               ▼
        ┌───────────────────────┐   ┌────────────────────┐         ┌────────────────────┐
        │   Rust Server (Axum)  │   │  Web client (JS)   │         │   Mobile (Flutter) │
        │  REST API + Leptos    │◄──┤  hydrates SSR;     │         │  local-first;      │
        │  SSR web app + Swagger│   │  WASM for sheet    │         │  sheet math via    │
        │  + static assets      │   │  math              │         │  flutter_rust_     │
        └───────────┬───────────┘   └────────────────────┘         │  bridge (FFI)      │
                    │ serves the web client and ◄── HTTP/REST ──────┤  optional sync     │
                    ▼                                               └────────────────────┘
            ┌─────────────────┐
            │     SQLite      │
            │    Database     │
            └─────────────────┘
```

The repository is a Cargo workspace plus the Flutter app under `mobile/`:

| Path | Crate / role |
|------|--------------|
| `server/` | `lorewyld` — Axum server: REST API **and** the Leptos SSR web app, in one binary |
| `shared/types/` | `lorewyld-types` — schema: every D&D 5e SRD content type + bundle/import format (see [DATA_MODEL.md](./DATA_MODEL.md)) |
| `shared/domain/` | `lorewyld-domain` — pure 5e rules math (and the home for future authoring validation) |
| `shared/domain-wasm/` | `lorewyld-domain-wasm` — `wasm-bindgen` wrapper compiled to WASM for the web client |
| `mobile-ffi/` | `lorewyld_mobile_ffi` — `flutter_rust_bridge` bridge exposing the core to Flutter |
| `tools/bundle-gen/` | Generates the shipped SRD content bundle from Open5e |
| `mobile/` | Flutter app (iOS / Android) |

`shared/domain-wasm` and `mobile-ffi` build `cdylib` artifacts for non-host
targets (WASM, iOS/Android) and are intentionally excluded from the workspace
`default-members`.

## Shared Rust core

The defining architectural decision: the schema and the 5e rules engine live
once, in pure Rust (no async/runtime dependencies), and are consumed three ways.

- **Natively** by the server and the Leptos SSR web app (both are Rust).
- **As WASM** by the web client's JavaScript — `lw-content.js` async-imports the
  module and backs `abilityMod` / `proficiencyBonus` / `deriveStats`, gating
  render on readiness. Built by `scripts/regen-wasm.sh` into `server/assets/wasm/`.
- **Over FFI** by Flutter via `flutter_rust_bridge` (a `#[frb(sync)]` surface,
  initialized with `RustLib.init` in `main.dart`). Built by `scripts/regen-ffi.sh`.

Because sheet math is one compiled crate, the server, web, and mobile cannot
disagree about how a modifier or proficiency bonus is computed. A contract test
(`server/src/content.rs::compendium_contract`) additionally guards the
client-consumed content field names against schema renames.

> An earlier iteration generated Dart types from Rust via `typeshare`; that
> codegen has been removed in favor of the FFI/WASM core. Mobile's Dart data
> types in `mobile/lib/types/` are now hand-written mirrors of the schema.

The whole workspace sets `unsafe_code = "forbid"`; only the two codegen crates
(`mobile-ffi`, `domain-wasm`) opt out, since their sole `unsafe` comes from
generated bridge code.

## Backend

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Runtime | Tokio | Async I/O and task scheduling |
| Web framework | Axum 0.8 | HTTP routing and middleware |
| Web UI | Leptos 0.8 (SSR) | Server-rendered web client, served by the same binary |
| API docs | utoipa + Swagger UI | OpenAPI spec at `/api/openapi.json`, interactive docs at `/swagger-ui` |
| Database | SQLite | Data persistence |
| DB driver | SQLx 0.8 | Async access, compile-time query checking, migrations |
| Serialization | Serde | JSON serialization/deserialization |
| Auth | argon2 (hashing) + opaque session tokens | Password auth, Bearer-token sessions |
| Error handling | anyhow | Flexible error handling |
| CLI | Clap | Command-line argument parsing |
| Config | config (TOML) | File-based configuration |
| Logging | tracing | Structured logging and diagnostics |

### Design Principles

1. **Single-source the rules.** 5e math and schema live in the shared Rust core;
   every client consumes it natively, via WASM, or via FFI — never reimplements it.
2. **Document and compute; never enforce or block.** Sheet math auto-applies
   declared rules, but the tool never gates a player's choice. No validation
   walls, no compatibility checks, no approval workflows.
3. **Content replicates; module IDs attribute.** Every content record carries a
   `content_module_uuid` for provenance, licensing UI, attribution, and search
   filters — it never prevents copying.
4. **Self-hostable & offline-first.** Run anywhere from a cloud VM to a Raspberry
   Pi; the mobile app works fully offline and syncs when connected.
5. **Portable state.** SQLite means a single file is the entire database — back
   up, restore, or migrate by copying it.

### Database

SQLite (via SQLx) was chosen for simplicity, portability, easy backup, and
performance that is more than sufficient for small-group gaming. Schema changes
are managed as SQLx migrations under `server/migrations/`.

#### Schema Conventions

- **Identifiers** are UUIDs stored as `TEXT`.
  - Content records use a column named `uuid` whose value is a **deterministic
    UUIDv5** derived from `{type}:{key}` (see [DATA_MODEL.md](./DATA_MODEL.md)),
    so regenerating a bundle never churns identities and foreign keys are
    computable without lookups.
  - User/session/app records use random UUIDs.
- **Timestamps** (`created_at` / `updated_at`) are `TEXT` populated by SQLite's
  `datetime('now')` and mapped to `chrono` types on the Rust side.
- **Booleans** are stored as `INTEGER` (`0` / `1`).
- Foreign keys and unique constraints are named explicitly.

The schema is defined by the Rust types in `shared/types/` (the single source of
truth) and the migrations that mirror them; the OpenAPI document describes the
API-facing shapes.

### API Design

The API follows REST conventions: JSON request/response bodies, standard HTTP
methods, and meaningful status codes. The full surface is documented via OpenAPI
(`/api/openapi.json`, browsable at `/swagger-ui`). Resource areas include
auth/users, characters, compendium (content browse), lore notes, tags, settings,
modules (install/manage), search, and admin endpoints.

**Authentication:** registration is gated by a per-server **join code**.
Passwords are hashed with **argon2id**. A successful register/login returns an
opaque **session token**; clients present it as `Authorization: Bearer <token>`,
and the server resolves it against the `user_session` table on each request.
Logout revokes the token (idempotent).

## Web Client (Leptos SSR)

The web client is not a separate service — it is rendered by the same Axum
binary, which merges the Leptos route handlers with the `/api/*` routes and
serves static assets (including the WASM module) on one router. It provides a
full client: home feed, compendium browse, character sheets, lore authoring,
module management, server/user settings, a dice roller, and auth UI. Live sheet
math runs in the browser through the WASM build of `lorewyld-domain`.

## Mobile Client (Flutter)

### Technology Stack

| Component | Technology |
|-----------|------------|
| Framework | Flutter (iOS, Android) |
| Local storage | sqflite |
| Rules math | `lorewyld-domain` via `flutter_rust_bridge` (FFI) |

### Local-first model

The mobile app is fully usable offline; a server connection is optional. `sqflite`
is the content home (characters, settings, lore notes), and all 5e sheet math is
computed by the shared Rust core over FFI rather than reimplemented in Dart. The
app shell is always available; signing in to a server unlocks sync and module
browsing.

### Sync

```
Mobile Device                    Server
┌───────────┐                ┌───────────┐
│  Local    │    pull        │  Server   │
│  Storage  │◄───────────────│  Database │
│           │    push        │           │
│           │───────────────►│           │
└───────────┘                └───────────┘
```

- **Settings and their lore notes** sync per-setting with last-write-wins
  semantics: push creates/updates server rows (mapping local↔remote UUIDs on
  first push), pull upserts local copies. Deletions do not propagate.
- **Modules** are browsed and installed from the server.
- The **Promote-to-Module** flow publishes a setting's notes as a shareable,
  attributed module package.

## Content & Bundles

Base content follows the D&D 5e SRD (CC BY 4.0), aligned with the Open5e v2 API.
Content is packaged as a `ContentBundle` — a versioned, self-describing JSON
package importable via the admin module-install endpoint. The bundle format,
type model, provenance graph, and generator are documented in
[DATA_MODEL.md](./DATA_MODEL.md). See also
[decisions/001-database-choice.md](./decisions/001-database-choice.md).

## Deployment

A single binary serves the API and web app. It exposes a `server` subcommand
(plus configuration via a TOML file); on startup it runs migrations, ensures the
game-server record, and seeds the embedded SRD content. A `Dockerfile` and
`docker-compose.yml` are provided for containerized hosting.

### Recommended Configurations

**Home Network (Raspberry Pi)**
- Minimal resource requirements
- Access via local IP or home DNS
- Perfect for regular gaming groups

**Cloud Hosting**
- Always accessible
- Good for distributed groups
- Any VPS or container platform works

### Data Management

```bash
# Backup
cp data.db data.db.backup

# Restore
cp data.db.backup data.db

# Share with another server
scp data.db user@other-server:/path/to/
```
