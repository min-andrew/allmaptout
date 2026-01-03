# Allmaptout

A full-stack web application template with Rust backend, Astro frontend, and Kubernetes deployment.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Astro, Svelte 5, Tailwind CSS |
| Backend | Rust, Axum, SQLx |
| Database | PostgreSQL 16 |
| Container | Docker |
| Orchestration | Kubernetes |
| CI/CD | GitHub Actions |

## Features

- Hot reload development for both frontend and backend
- Type-safe API client generation from OpenAPI
- Database migrations at startup
- Rate limiting and security headers
- Structured JSON logging in production
- Graceful shutdown handling
- Health check endpoints
- Non-root Docker containers
- Kubernetes-ready with health probes and resource limits

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) + pnpm (`npm install -g pnpm`)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [just](https://github.com/casey/just) (command runner)

### Install (macOS)

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node tools
npm install -g pnpm

# Docker
brew install --cask docker

# Command runner
brew install just

# Git hooks
brew install lefthook
```

## Quick Start

```bash
# Clone
git clone https://github.com/your-username/allmaptout.git
cd allmaptout

# Create .env from template
cp .env.example .env

# Generate secrets and update .env
openssl rand -base64 32  # Use for JWT_SECRET
# Edit .env with your values

# Install Rust tools
rustup component add clippy rustfmt
cargo install cargo-watch sqlx-cli

# Install frontend dependencies
cd frontend && pnpm install && cd ..

# Install git hooks
lefthook install

# Start database
docker compose up -d db

# Run migrations
just db-migrate
```

## Development

```bash
# Terminal 1: Backend (http://localhost:3001)
just backend

# Terminal 2: Frontend (http://localhost:4321)
just frontend

# Terminal 3 (optional): Watch tests
just watch-tests
```

## Commands

```bash
# Development
just backend        # Backend with hot reload
just frontend       # Frontend with HMR
just watch-tests    # Run tests on file changes

# Quality
just test           # Run all tests
just lint           # Lint all code
just format         # Format all code
just check          # Run all checks (CI)

# Database
just db-start       # Start postgres
just db-stop        # Stop postgres
just db-migrate     # Run migrations

# Code Generation
just generate       # Generate frontend API hooks from OpenAPI
```

## Project Structure

```
allmaptout/
├── backend/                # Rust API
│   ├── src/
│   │   ├── main.rs         # Entry point, migrations, server
│   │   ├── lib.rs          # Router, handlers
│   │   ├── config.rs       # Environment config
│   │   └── error.rs        # Error types
│   ├── migrations/         # SQL migrations
│   ├── Cargo.toml
│   ├── rust-toolchain.toml # Pinned Rust version
│   └── Dockerfile
├── frontend/               # Astro site
│   ├── src/
│   │   ├── pages/          # Routes
│   │   ├── layouts/        # Page templates
│   │   ├── components/     # Svelte components
│   │   └── api/            # API client
│   ├── package.json
│   └── Dockerfile
├── k8s/                    # Kubernetes manifests
│   ├── backend.yaml
│   ├── frontend.yaml
│   ├── ingress.yaml
│   ├── cert-manager.yaml   # TLS certificates
│   └── ingress-tls.yaml.example
├── .github/workflows/      # CI/CD
├── justfile                # Task runner
├── lefthook.yml            # Git hooks
└── docker-compose.yml      # Local postgres
```

## Environment Variables

Copy `.env.example` to `.env` and configure:

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `JWT_SECRET` | Secret for JWT signing | Yes |
| `PORT` | Backend port (default: 3001) | No |
| `RUST_ENV` | `development` or `production` | No |
| `RUST_LOG` | Log level (default: info) | No |
| `CORS_ORIGIN` | Frontend URL for CORS (production only) | Prod |

## Deployment

### Docker Build

```bash
# Build images
docker build -t myapp-backend ./backend
docker build -t myapp-frontend ./frontend
```

### Kubernetes

1. Create secrets:
```bash
kubectl create secret generic app-secrets \
  --from-literal=DATABASE_URL='postgres://...' \
  --from-literal=JWT_SECRET='...' \
  --from-literal=CORS_ORIGIN='https://your-domain.com'
```

2. Apply manifests:
```bash
kubectl apply -f k8s/
```

3. For HTTPS, install cert-manager and configure:
```bash
# Install cert-manager
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.14.4/cert-manager.yaml

# Update email in cert-manager.yaml, then apply
kubectl apply -f k8s/cert-manager.yaml

# Copy and configure TLS ingress
cp k8s/ingress-tls.yaml.example k8s/ingress-tls.yaml
# Edit YOUR_DOMAIN in ingress-tls.yaml
kubectl apply -f k8s/ingress-tls.yaml
```

### CI/CD

GitHub Actions automatically:
1. Runs linting and tests on all PRs
2. Builds and pushes Docker images on main
3. Deploys to Kubernetes (requires `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` secrets)

## Adding API Endpoints

1. Add endpoint in `backend/src/lib.rs` with utoipa annotations:
```rust
#[utoipa::path(get, path = "/items", responses((status = 200, body = Vec<Item>)))]
pub async fn list_items() -> Json<Vec<Item>> { ... }
```

2. Generate frontend hooks:
```bash
just generate
```

3. Use in frontend:
```typescript
import { listItems } from '../api/generated';
const items = await listItems();
```

## License

MIT
