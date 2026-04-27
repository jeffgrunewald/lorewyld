# Architecture

## System Overview

Lorewyld follows a client-server architecture with a REST API backend and mobile client.

```
┌─────────────────┐         ┌─────────────────┐
│    Lorewyld     │  HTTP   │    Lorewyld     │
│  (Flutter App)  │◄───────►│  (Rust Server)  │
│                 │  REST   │                 │
└─────────────────┘         └────────┬────────┘
                                     │
                                     ▼
                            ┌─────────────────┐
                            │     SQLite      │
                            │    Database     │
                            └─────────────────┘
```

## Backend

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Runtime | Tokio | Async I/O and task scheduling |
| Web Framework | Axum | HTTP routing and middleware |
| Database | SQLite | Data persistence |
| DB Driver | SQLx | Async database access with compile-time query checking |
| Serialization | Serde | JSON serialization/deserialization |
| Error Handling | anyhow | Flexible error handling |
| CLI | Clap | Command-line argument parsing |
| Logging | tracing | Structured logging and diagnostics |

### Design Principles

1. **Simplicity**: SQLite for single-file database management
2. **Portability**: Easy to backup, restore, and migrate by copying the database file
3. **Self-hostable**: Run anywhere from cloud VMs to Raspberry Pis
4. **Offline-first**: Mobile app works offline, syncs when connected

### Database

SQLite was chosen for:

- **Simplicity**: No separate database server to manage
- **Portability**: Single file contains all state
- **Backup**: Copy the file to backup; sync it to share
- **Performance**: More than sufficient for small-group gaming

#### Schema Conventions

All database tables follow these conventions:

- `id`: UUIDv4 stored as TEXT (primary key)
- `created_at`: ISO 8601 timestamp as TEXT
- `updated_at`: ISO 8601 timestamp as TEXT

See [shared/schemas/](../shared/schemas/) for JSON Schema definitions.

### API Design

The API follows REST conventions:

- JSON request/response bodies
- Standard HTTP methods (GET, POST, PUT, DELETE)
- Meaningful HTTP status codes
- Token-based authentication

## Mobile Client (Lorewyld)

### Technology Stack

| Component | Technology |
|-----------|------------|
| Framework | Flutter |
| Platforms | iOS, Android |

### Features

- Connect to game server by IP or DNS name
- Pull characters from server
- Push characters from local backup
- Offline character sheet access
- Campaign browsing and management

### Sync Model

```
Mobile Device                    Server
┌───────────┐                ┌───────────┐
│  Local    │    pull        │  Server   │
│  Storage  │◄───────────────│  Database │
│           │    push        │           │
│           │───────────────►│           │
└───────────┘                └───────────┘
```

Characters and campaign data can be:
- **Pulled**: Download from server to device
- **Pushed**: Upload from device to server (restore from backup)

## Deployment

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
