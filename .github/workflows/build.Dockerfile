FROM --platform=$TARGETPLATFORM rust:1.78-slim-bookworm

ARG RUST_TARGET
ENV RUST_TARGET=${RUST_TARGET}

RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

COPY . .

RUN rustup target add ${RUST_TARGET} && \
    cargo build --release --target ${RUST_TARGET} && \
    mkdir -p /output && \
    cp target/${RUST_TARGET}/release/ghost-resend-mailer /output/

CMD ["cp", "-r", "/output", "/out"] 