.PHONY: install install-frontend install-backend build build-frontend build-backend dev dev-frontend dev-backend run test test-frontend test-backend clean

# Install dependencies
install: install-frontend install-backend

install-frontend:
	cd frontend && npm ci

install-backend:
	cargo fetch

# Build
build: build-frontend build-backend

build-frontend:
	cd frontend && npm run build

build-backend:
	cargo build --release

# Development (run both concurrently)
dev:
	@echo "Starting frontend and backend in development mode..."
	@trap 'kill 0' EXIT; \
	cd frontend && npm run dev & \
	cargo run & \
	wait

dev-frontend:
	cd frontend && npm run dev

dev-backend:
	cargo run

# Production run (builds frontend first, then runs backend)
run: build-frontend
	cargo run

# Testing
test: test-backend test-frontend

test-backend:
	cargo test

test-frontend:
	cd frontend && npm run test

test-e2e:
	cd frontend && npm run test:e2e

# Clean
clean:
	cargo clean
	cd frontend && rm -rf dist node_modules
