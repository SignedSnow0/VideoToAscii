# --- Stage 1: Build the assets ---
FROM rust:slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    & rm -rf /var/lib/apt/lists/*

# Install WASM target and Trunk
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked trunk

WORKDIR /app
COPY . .

# Build assets in release mode (creates the /app/dist folder)
RUN trunk build --release

# --- Stage 2: Serve via Nginx ---
FROM nginx:alpine

# Copy the compiled production assets to Nginx
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
