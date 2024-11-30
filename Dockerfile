####################################################################################################
## Builder
####################################################################################################
FROM docker.io/library/rust:1.78-slim-bookworm AS builder

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create a dummy project to cache dependencies
RUN cargo new --bin dummy
WORKDIR /app/dummy

# Copy only the dependency files first
COPY Cargo.toml Cargo.lock ./

# Update the dummy main.rs to use our actual dependencies
RUN sed -i 's/name = "dummy"/name = "ghost-resend-mailer"/' Cargo.toml

# Build dependencies only
RUN cargo build --release
RUN rm src/*.rs

# Copy the actual source code
COPY ./src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Move the built binary to a known location
RUN cp target/release/ghost-resend-mailer /app/ghost-resend-mailer

####################################################################################################
## Final image
####################################################################################################
FROM docker.io/library/debian:bookworm-slim

# Install required packages
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    openssl \
    curl \
    libc6 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy our build
COPY --from=builder /app/ghost-resend-mailer /app/ghost-resend-mailer
RUN chmod +x /app/ghost-resend-mailer

# Environment variables
ENV RUST_LOG_STYLE=never
ENV RUST_LOG=info

# Expose the port the app runs on
EXPOSE 3000

CMD ["/app/ghost-resend-mailer"] 
