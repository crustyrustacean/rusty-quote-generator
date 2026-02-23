# Stage 1: Chef - prepare recipe
FROM rust:1.93.1 AS chef

RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner - create recipe.json
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY common ./common
COPY frontend ./frontend

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build frontend with Trunk
FROM rust:1.93 AS frontend-builder

RUN cargo install trunk
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY frontend ./frontend
COPY backend ./backend

WORKDIR /app/frontend
RUN trunk build --release

# Stage 4: Build backend with cached dependencies
FROM chef AS backend-builder

COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer is cached
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY frontend ./frontend

WORKDIR /app/backend
RUN cargo build --release

# Stage 5: Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /app/target/release/rusty-quote-generator-server /app/server
COPY --from=frontend-builder /app/public /app/public
COPY backend/configuration /app/configuration

ENV APP_ENVIRONMENT=production

EXPOSE 8080

CMD ["./server"]