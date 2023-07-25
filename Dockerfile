# Rust 1.71.0 is latest as of July 24, 2023
FROM rust:1.71.0 as builder

# Install the targets
RUN rustup target add $(arch)-unknown-linux-musl && \
	apt update && \
	apt install -y musl-tools

WORKDIR /app

COPY . ./

# Use a statically linked target for the prod
RUN cargo build -p janus-server --release --target $(arch)-unknown-linux-musl

# We want the health check to be minimal, and performance isn't a big concern for it
RUN cargo build -p healthcheck --release --target $(arch)-unknown-linux-musl

# Coalesce all the compiled binaries into a final directory for each output
# so it's easier to copy in the next stage
RUN mkdir ./bin && \
	mv ./target/$(arch)-unknown-linux-musl/release/janus-server \
	./target/$(arch)-unknown-linux-musl/release/healthcheck  ./bin

# Prevent reading+writing to the binaries, making them execute-only
RUN chmod -rw ./bin/* && \
	chown -R 1000:1000 ./bin

# Create a debug container with things like a shell and package manager for additional
# tools.
FROM alpine:latest as patched

RUN apk update && apk upgrade --no-cache

COPY --from=builder /app/bin/ /

ENV JANUS_LOG_LEVEL info
ENV JANUS_PORT 8000
ENV JANUS_CONFIG_FILE janus.yaml

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/janus-server"]

# Can use other distroless containers if desired
FROM scratch as prod

COPY --from=builder /app/bin/ /

ENV JANUS_LOG_LEVEL info
ENV JANUS_PORT 8000
ENV JANUS_CONFIG_FILE janus.yaml

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/janus-server"]