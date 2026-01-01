# Wedding App

## Setup

```bash
# Copy environment file and edit with your values
cp .env.example .env

# Install tools and dependencies
just setup

# Start database
just db-start

# Start development
just dev-backend   # Terminal 1
just dev-frontend  # Terminal 2
```

## Commands

```bash
just dev-backend    # Start backend with hot reload
just dev-frontend   # Start frontend with HMR
just watch-tests    # Run tests on file changes

just test           # Run all tests
just lint           # Lint all code
just format         # Format all code
just check          # Run all checks (CI equivalent)

just generate-api   # Generate frontend hooks from backend OpenAPI
```

## Structure

```
├── backend/        # Rust + Axum
├── frontend/       # Astro + Svelte + Tailwind
├── justfile        # Commands
└── lefthook.yml    # Git hooks
```

## Environment

Copy `.env.example` to `.env` and fill in your values. Never commit `.env`.
