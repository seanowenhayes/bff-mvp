# BFF MVP

Backend-for-Frontend MVP with Axum - a proxy layer with request/response capture and GUI for route management.

## Overview

This MVP provides:
- **Reverse proxy** to an existing BFF (Backend-for-Frontend)
- **Request/response capture** for all proxied traffic
- **REST API** for route management and log viewing
- **In-memory storage** (routes & logs)
- Ready for a React GUI to manage routes and view captured traffic

## Architecture

```
[Client] → [BFF MVP:8080] → [Original BFF:3000]
                ↓
         Capture logs
         /api/routes (GET/POST)
         /api/logs (GET)
```

## Quick Start

### Prerequisites
- Rust 1.70+ ([Install](https://rustup.rs/))
- An existing BFF running (default: `http://localhost:3000`)

### Run

```bash
# Clone
git clone https://github.com/seanowenhayes/bff-mvp.git
cd bff-mvp

# Set target BFF URL (optional, defaults to http://localhost:3000)
export TARGET_BFF_URL=http://localhost:3000

# Run
cargo run
```

The server starts on `http://0.0.0.0:8080`.

## API Endpoints

### GET /api/routes
Returns all configured routes.

**Response:**
```json
[
  {
    "id": 1,
    "path": "/users",
    "method": "GET",
    "mode": "proxy",
    "target_path": null,
    "description": "User list endpoint"
  }
]
```

### POST /api/routes
Create or update a route.

**Request:**
```json
{
  "id": 1,
  "path": "/users",
  "method": "GET",
  "mode": "proxy",
  "target_path": null,
  "description": "User list endpoint"
}
```

### GET /api/logs
Returns captured request/response logs (last 1000).

**Response:**
```json
[
  {
    "timestamp": "2025-12-17T22:00:00Z",
    "method": "GET",
    "path": "/users",
    "status": 200,
    "latency_ms": 45,
    "request_body": null,
    "response_body": "{\"users\":[...]}"
  }
]
```

### All other paths
Proxied to the target BFF and logged.

## Configuration

- **TARGET_BFF_URL**: Environment variable for the original BFF (default: `http://localhost:3000`)
- **Port**: Hardcoded to `8080` (change in `main.rs`)

## Next Steps

1. **PostgreSQL**: Replace in-memory storage with SQLx models
2. **React GUI**: Build a dashboard to:
   - View/edit routes
   - Toggle proxy/handled mode per route
   - View captured traffic per route
3. **Route matching**: Implement pattern matching for dynamic routes
4. **Response mocking**: Add "handled" mode to return mocked responses

## Development

```bash
# Check
cargo check

# Build
cargo build --release

# Run with logs
RUST_LOG=info cargo run
```

## License

MIT
