# Wedding App - Monorepo Commands
set dotenv-load := true

# Show available commands
default:
    @just --list

# ─────────────────────────────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────────────────────────────

# Start backend with hot reload
dev-backend:
    cd backend && cargo watch -x run

# Start frontend with HMR
dev-frontend:
    cd frontend && pnpm dev

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

# Run all checks (CI equivalent)
check:
    just lint
    just format-check
    just test

# Lint all code
lint:
    cd backend && cargo clippy --all-targets -- -D warnings
    cd frontend && pnpm lint

# Format all code
format:
    cd backend && cargo fmt
    cd frontend && pnpm format

# Check formatting
format-check:
    cd backend && cargo fmt --check
    cd frontend && pnpm format:check

# ─────────────────────────────────────────────────────────────────────────────
# Database
# ─────────────────────────────────────────────────────────────────────────────

# Start database
db-start:
    docker compose up -d db

# Stop database
db-stop:
    docker compose down

# Run migrations
db-migrate:
    cd backend && sqlx migrate run

# ─────────────────────────────────────────────────────────────────────────────
# Code Generation
# ─────────────────────────────────────────────────────────────────────────────

# Generate OpenAPI spec from backend
generate-openapi:
    cd backend && cargo run --bin openapi > ../frontend/openapi.json

# Generate frontend API hooks from OpenAPI
generate-api:
    just generate-openapi
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
