FROM rust:alpine AS builder
RUN apk update && apk upgrade && apk add lld musl musl-dev clang git make ca-certificates

WORKDIR /app

# Thanks https://github.com/rust-lang/cargo/issues/2644#issuecomment-3819570767
# Copy the cargo lock and build the deps first, so that Docker can cache them
COPY Cargo.lock Cargo.toml ./
RUN mkdir src; echo 'fn main() {}' >src/main.rs
RUN cargo rustc --release

# Then build the rest of it
COPY src ./src/
RUN cargo build --release

FROM alpine:latest

RUN apk update && apk upgrade && apk add ca-certificates tzdata

RUN update-ca-certificates

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "1001" \
    "oxide"

COPY --from=builder /app/target/release/oxide_server /bin/oxide_server

USER oxide:oxide
ENTRYPOINT ["/bin/oxide_server"]
