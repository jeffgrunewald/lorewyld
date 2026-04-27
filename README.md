# Lorewyld

A self-hostable web server for storing, sharing, and building role-playing campaigns. Designed to facilitate easy in-person gaming experiences with friends.

## Overview

Lorewyld is a self-hostable backend service that manages campaign data, characters, and game resources. It pairs with a companion Flutter mobile app (also named **Lorewyld**) for interacting with the server during gaming sessions.

## Architecture

- **Backend**: Rust application built on the Tokio async runtime with Axum web framework
- **API**: RESTful API for all client interactions
- **Database**: SQLite via SQLx for simplicity and portability
- **Mobile**: Lorewyld mobile — Flutter application for iOS and Android

## Key Features

- Store and manage role-playing campaigns
- Share characters and game data across devices
- Simple state management - backup by copying the SQLite file
- Sync characters between mobile devices and server
- Connect to game servers by IP address or DNS name

## Deployment Options

Lorewyld is designed to run anywhere:

- **Cloud hosting**: Deploy to any hosting provider
- **Home network**: Run on a Raspberry Pi or home server
- **Local**: Run on your laptop for local-only sessions

## Getting Started

### Prerequisites

- Rust (edition 2024)
- SQLite

### Running the Server

```bash
cd backend
cargo run
```

The server will start on `http://localhost:3000`.

### Mobile App

See the [mobile/](mobile/) directory for the Lorewyld mobile app setup instructions.

## Project Structure

```
lorewyld/
├── backend/          # Rust web server
│   ├── src/          # Application source code
│   └── migrations/   # SQLx database migrations
├── mobile/           # Lorewyld mobile Flutter app
├── shared/           # Shared schemas and API specs
│   ├── schemas/      # JSON Schema definitions
│   └── api-spec/     # API specifications
└── docs/             # Documentation
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/API.md)
- [Development Guide](docs/DEVELOPMENT.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
