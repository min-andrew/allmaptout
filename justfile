# allmaptout
set dotenv-load := true

# Show available commands
default:
    @just --list

# ─────────────────────────────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────────────────────────────

# Start backend with hot reload
backend:
    cd backend && cargo watch -x run

# Start frontend with HMR + type checking
frontend:
    cd frontend && pnpm dev:check

# Watch and run tests on file changes
watch-tests:
    cd backend && bacon test

# ─────────────────────────────────────────────────────────────────────────────
# Testing
# ─────────────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cd backend && cargo nextest run
    cd frontend && pnpm test

# Run backend tests
test-backend *ARGS:
    cd backend && cargo nextest run {{ARGS}}

# Run frontend tests
test-frontend:
    cd frontend && pnpm test

# ─────────────────────────────────────────────────────────────────────────────
# Code Quality
# ─────────────────────────────────────────────────────────────────────────────

# Run all checks
check:
    just format
    just lint
    cd frontend && pnpm typecheck
    just test

# Lint all code
lint:
    cd backend && cargo clippy --all-targets -- -D warnings
    cd frontend && pnpm lint

# Format all code
format:
    cd backend && cargo fmt
    cd frontend && pnpm format

# ─────────────────────────────────────────────────────────────────────────────
# Database
# ─────────────────────────────────────────────────────────────────────────────

# Start database
db-start:
    docker compose up -d db

# Start database with GUI (http://localhost:8080)
db-gui:
    docker compose up -d db adminer
    @echo "Database GUI available at http://localhost:8080"

# Stop database
db-stop:
    docker compose down

# Run migrations
db-migrate:
    cd backend && sqlx migrate run

# ─────────────────────────────────────────────────────────────────────────────
# Code Generation
# ─────────────────────────────────────────────────────────────────────────────

# Generate frontend api hooks from backend Openapi
generate:
    cd backend && cargo run --bin openapi > ../frontend/openapi.json
    cd frontend && pnpm generate

# ─────────────────────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────────────────────

# Initial setup
setup:
    @echo "Installing Rust tools..."
    rustup component add clippy rustfmt
    cargo install cargo-watch cargo-nextest bacon sqlx-cli
    @echo "Installing git hooks..."
    lefthook install
    @echo "Installing frontend dependencies..."
    cd frontend && pnpm install
    @echo "✓ Setup complete"
