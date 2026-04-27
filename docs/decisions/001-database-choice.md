# ADR 001: SQLite as the Database

## Status

Accepted

## Context

Lorewyld needs a database to store campaigns, characters, and user data. The application is designed for individual gaming groups to self-host, typically supporting a small number of concurrent users (rarely more than a dozen players in a session).

Key requirements:
- Simple deployment without external dependencies
- Easy backup and restore
- Portable across hosting environments (cloud VMs, Raspberry Pi, laptops)
- Sufficient performance for small concurrent user counts

## Decision

Use **SQLite** as the database, accessed via the **SQLx** crate.

## Rationale

### Simplicity
- No separate database server to install, configure, or maintain
- Single dependency bundled with the application
- Reduces operational complexity for self-hosters

### Portability
- Entire database is a single file
- Easy to move between environments
- Works identically on cloud servers, home networks, and Raspberry Pis

### Ease of Maintenance
- **Backup**: Copy the database file
- **Restore**: Replace the database file
- **Sync/Replicate**: Use standard file sync tools (rsync, rclone, etc.)
- Power users can set up automated replication with existing tooling

### Performance
- More than sufficient for the expected load (< 12 concurrent users)
- SQLite handles hundreds of concurrent readers efficiently
- Write contention is minimal for turn-based gaming applications

## Alternatives Considered

### PostgreSQL
- Pros: More powerful, better concurrent write handling
- Cons: Requires separate server, more complex deployment, overkill for use case

### MySQL/MariaDB
- Pros: Widely supported, good tooling
- Cons: Same complexity issues as PostgreSQL, no significant benefit for this scale

### Embedded Key-Value (RocksDB, sled)
- Pros: High performance, embedded
- Cons: No SQL, more complex querying, less familiar to contributors

## Consequences

- Single-writer limitation is acceptable for this use case
- Database migrations managed via SQLx
- Backup documentation should emphasize file-based approach
- Future scaling (if ever needed) would require migration to PostgreSQL
