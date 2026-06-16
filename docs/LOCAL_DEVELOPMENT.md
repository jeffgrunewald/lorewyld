# Local Development

How to run the Lorewyld server and the Flutter mobile app side by side on one
machine, connect the app (in a simulator/emulator) to the server — including a
server running on the same host — and test pushing/pulling content.

See [ARCHITECTURE.md](./ARCHITECTURE.md) for how the pieces fit together.

## Prerequisites

- **Rust** (edition 2024 toolchain) — builds the server and, via
  [cargokit](https://github.com/irondash/cargokit), the mobile FFI native
  library automatically during a Flutter build.
- **Flutter** SDK, plus a simulator/emulator:
  - **iOS:** Xcode + an iOS Simulator (macOS only).
  - **Android:** Android Studio + an Android Virtual Device (AVD), or a physical
    device.
- **sqlite3** CLI — to read the server's join code.
- (Only when editing the Rust↔Dart bridge) `flutter_rust_bridge_codegen`
  matching the pinned `flutter_rust_bridge` version in `mobile/pubspec.yaml`
  (currently `2.12.0`). Not needed just to run the app.

Once the toolchains are installed, fetch dependencies and build the server once:

```bash
scripts/setup-dev.sh
```

The common workflows below are wrapped in scripts under `scripts/`; the raw
commands are shown alongside so you know what each does.

## 1. Run the server

The server is a single binary that serves the JSON API, the web UI, and Swagger.

```bash
scripts/run-backend.sh
# equivalent to: (cd server && cargo run -- -c config.toml server)
# LW_RELEASE=1 scripts/run-backend.sh   # release build
```

It runs from the `server/` directory so it picks up `server/config.toml` and
creates its database there.

On first start it runs migrations, creates the `game_server` record (generating
a **join code**), generates a JWT signing key, and seeds the embedded SRD
content. You should see it listening on `0.0.0.0:8080`.

Once running, on the host machine:

- Web UI: <http://localhost:8080>
- Swagger / OpenAPI: <http://localhost:8080/swagger-ui> (spec at
  `/api/openapi.json`)
- Default admin login: username `admin`, password `admin` (change it after first
  login)

### Why `0.0.0.0` matters

`config.toml` sets `api_addr = "0.0.0.0:8080"`, which binds **all** interfaces.
This is deliberate: an emulator/simulator (and physical devices) reach the
server over the host's network, not its loopback, so binding only `127.0.0.1`
would make the server unreachable from the device. Keep it on `0.0.0.0` for
local device testing.

### Get the join code (needed to register accounts)

```bash
sqlite3 server/database.db 'SELECT join_code FROM game_server'
```

### Configuration & overrides

Settings come from `server/config.toml`, overridable by `LW__`-prefixed env vars
(double-underscore separator). Defaults:

| Setting | `config.toml` key | Env var | Default |
|---|---|---|---|
| Listen address | `api_addr` | `LW__API_ADDR` | `0.0.0.0:8080` |
| Database path | `db_path` | `LW__DB_PATH` | `database.db` |
| Log filter | `log` | `LW__LOG` | `lorewyld=info` |
| JWT key path | `jwt_key_path` | `LW__JWT_KEY_PATH` | `jwt_signing_key.bin` |

```bash
# Example: run on a different port
LW__API_ADDR=0.0.0.0:9000 scripts/run-backend.sh
```

> **Reset the database** by stopping the server and deleting `server/database.db`
> (it is gitignored and reseeds on next start). Do this after content/schema
> migrations.

## 2. Run the mobile app in a simulator

```bash
scripts/run-mobile.sh          # choose your booted simulator/emulator when prompted
scripts/run-mobile.sh -d "iPhone 15"   # or target a specific device
# equivalent to: (cd mobile && flutter pub get && flutter run)
```

`run-mobile.sh` also prints the per-platform Server URL and the current join
code before launching, so you have what you need for step 3.

The first build compiles the shared Rust core into the app's native FFI library
(cargokit) and `RustLib.init()` wires it up at startup — no manual FFI build
step is required just to run.

## 3. Connect the app to the server

The app is local-first and fully usable offline. To connect: tap the **cloud
icon** in the app bar → opens the **Server** screen → choose **Log in** or
**Register**, fill in the **Server URL**, then your credentials.

**The Server URL depends on where the app runs**, because each simulator/emulator
reaches the host machine through a different address:

| App runs on | Server URL to enter | Why |
|---|---|---|
| **iOS Simulator** | `http://localhost:8080` (or `http://127.0.0.1:8080`) | The simulator shares the host's network stack, so loopback *is* the host. |
| **Android Emulator** | `http://10.0.2.2:8080` | The emulator runs in a VM; `10.0.2.2` is its special alias for the host's loopback. `localhost` would point at the emulator itself. |
| **Physical device** (same Wi-Fi) | `http://<host-LAN-IP>:8080` | The device must reach the host over the LAN. Find the host IP with `ipconfig getifaddr en0` (macOS). |

> The app's Server URL field defaults to `http://10.0.2.2:8080` (the Android
> emulator case). **On the iOS Simulator, change it to `http://localhost:8080`.**

### Register vs. log in

- **Register** (first time): Server URL, **join code** (from step 1), username,
  email, password.
- **Log in**: Server URL, username, password.

A successful register/login stores an opaque session token; the app sends it as
`Authorization: Bearer <token>` and shows a connected (cloud) state.

## 4. Test pushing and pulling content

Content sync is **per-setting** with last-write-wins semantics. A typical loop:

1. **Create content locally** (works offline): make a **Setting**, then add one
   or more **Lore Notes** to it.
2. **Connect** to the server (step 3) — register with the join code or log in.
3. **Push**: from the setting list, trigger sync/push for the setting. The first
   push creates the server-side setting and notes and links local↔remote UUIDs;
   re-pushing updates rather than duplicating.
4. **Verify on the server**: open the web UI (<http://localhost:8080>) and browse
   the setting / lore notes, or inspect via Swagger (`GET /api/settings`,
   `GET /api/lore-notes`).
5. **Pull**: edit a note on the web (or from another device), then use the
   pull/download action on the setting list in the app. Local copies are upserted
   from the server (last-write-wins); local-only notes are kept.
6. **Publish** (optional): use the **Promote-to-Module** flow to publish a
   setting's notes as a shareable, attributed module package, then browse/install
   it from another account.

**Behavior to expect:**

- Deletions do **not** propagate (push/pull never deletes).
- Characters and character-scoped notes are local-only (not synced).
- Conflict handling is last-write-wins; there is no merge/conflict UI.

### End-to-end test (no simulator needed)

A host-side e2e test exercises register → push → re-push → server-edit → pull →
publish → logout against a live server:

```bash
# with the server running:
cd mobile
LW_E2E=1 LW_JOIN_CODE=$(sqlite3 ../server/database.db 'SELECT join_code FROM game_server') \
  flutter test test/sync_e2e_test.dart
```

This runs on the Dart VM and talks real HTTP, so it bypasses the platform
networking restrictions noted below — useful to confirm the server side
independently of simulator setup.

## Troubleshooting

**"Connection refused" / request times out**
- Confirm the server is up and bound to `0.0.0.0:8080` (not `127.0.0.1`).
- Confirm you used the right host alias for your platform (table above) —
  `10.0.2.2` for the Android emulator, `localhost` for the iOS Simulator.
- For a physical device, confirm host and device are on the same network and no
  firewall blocks port 8080.

**Cleartext HTTP blocked (Android emulator).** Local dev uses plain `http://`,
but Android blocks cleartext traffic by default on API 28+. If connections fail
with a cleartext error, allow it for debug builds — e.g. add to the
`<application>` element in `mobile/android/app/src/debug/AndroidManifest.xml`:

```xml
<application android:usesCleartextTraffic="true" ... >
```

(or attach a `network_security_config.xml` permitting `10.0.2.2`). Prefer scoping
this to the debug manifest so it never ships in release.

**App Transport Security blocks HTTP (iOS Simulator).** iOS ATS blocks arbitrary
`http://` by default. If the iOS Simulator can't reach the local HTTP server, add
an ATS exception to `mobile/ios/Runner/Info.plist`:

```xml
<key>NSAppTransportSecurity</key>
<dict>
  <key>NSAllowsLocalNetworking</key>
  <true/>
</dict>
```

**401 / session dropped after restart.** The app probes `/api/users/me` on load
and drops an invalid/expired session (a network error is treated as "offline" and
the session is kept). Log in again from the Server screen.

**Wrong join code on register.** Re-read it with the `sqlite3` command in step 1;
it changes if the database is reset.
