# Allmaptout

## Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/) + pnpm (`npm install -g pnpm`)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
- [just](https://github.com/casey/just)

### Install prerequisites (macOS)
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

## Setup
```bash
# Clone
git clone git@github.com:min-andrew/allmaptout.git
cd allmaptout

# Create .env from template
cp .env.example .env

# Generate secrets and update .env
openssl rand -base64 16  # Use for DB_PASSWORD
openssl rand -base64 32  # Use for JWT_SECRET
nvim .env                # Paste the generated values

# Install Rust tools
rustup component add clippy rustfmt
cargo install cargo-watch cargo-nextest bacon sqlx-cli

# Install frontend dependencies
cd frontend && pnpm install && cd ..

# Install git hooks
lefthook install

# Start database
docker compose up -d db
```

## Development
```bash
# Terminal 1: Backend (http://localhost:3001)
just backend

# Terminal 2: Frontend (http://localhost:3000)
just frontend

# Terminal 3 (optional): Watch tests
just watch-tests
```

## Commands
```bash
just backend    # Backend with hot reload
just frontend   # Frontend with HMR
just watch-tests    # Run tests on file changes

just test           # Run all tests
just lint           # Lint all code
just format         # Format all code
just check          # Run all checks (CI)

just db-start       # Start postgres
just db-stop        # Stop postgres
just db-migrate     # Run migrations

just generate   # Generate frontend hooks from backend OpenAPI
```

## The personal editor setup for now (Neovim, but using LazyVim for now)
```bash
# Install LazyVim
mv ~/.config/nvim ~/.config/nvim.bak 2>/dev/null
git clone https://github.com/LazyVim/starter ~/.config/nvim
rm -rf ~/.config/nvim/.git
nvim  # First launch installs plugins

# In Neovim, install language servers
:MasonInstall rust-analyzer svelte-language-server astro-language-server typescript-language-server tailwindcss-language-server prettier eslint_d taplo
```

## Project structure
```
├── backend/           # Rust + Axum
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       └── config.rs
├── frontend/          # Astro + Svelte + Tailwind
│   └── src/
│       ├── pages/
│       ├── layouts/
│       └── api/
├── justfile           # Commands
├── lefthook.yml       # Git hooks
└── docker-compose.yml # Postgres
```

## Environment

Copy `.env.example` to `.env` and fill in real values. Never commit `.env`.

