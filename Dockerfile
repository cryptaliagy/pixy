# Rust 1.69.0 is latest as of Apr 20, 2023
FROM rust:1.69.0 as dev

# Install the targets
RUN rustup target add $(arch)-unknown-linux-musl $(arch)-unknown-linux-gnu

# Sets some basic environment variables for configuration of the web server.
# This is useful for using the `builder` image as the target for dev, which
# I don't generally recommend but can be useful under some specific circumstances
ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL sparse

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

# Create a fake binary target to be used for dependency caching locally, then clean it
RUN mkdir src && echo "fn main() {}" > src/main.rs \
	&& cargo new --bin healthcheck \
	&& cargo build \
	&& cargo test \
	&& rm src/main.rs

COPY src ./src

COPY healthcheck ./healthcheck

RUN touch -am src/main.rs \
	&& cargo test \
	&& cargo build \
	&& cp -r ./target/debug out


CMD ["./out/websvc"]

# Create a builder container to compile the optimized release version
# of the service. We don't care too much about how many layers get generated in this step 
FROM dev as builder

# Statically link the C runtime so we can still use `scratch` at the end, but side-steps some of the potential
# performance concerns of musl, if that is applicable. This exports as the `prod-gnu` target.
RUN RUSTFLAGS="-C target-feature=+crt-static" cargo build -p websvc --release --target $(arch)-unknown-linux-gnu

# Use a statically linked target for the prod
RUN cargo build -p websvc --release --target $(arch)-unknown-linux-musl

# We want the health check to be minimal, and performance isn't a big concern for it
RUN cargo build -p healthcheck --release --target $(arch)-unknown-linux-musl

# Coalesce all the compiled binaries into a final directory for each output
# so it's easier to copy in the next stage
RUN mkdir ./bin ./bin-gnu && \
	mv ./target/$(arch)-unknown-linux-musl/release/websvc \
	./target/$(arch)-unknown-linux-musl/release/healthcheck  ./bin \
	&& mv ./target/$(arch)-unknown-linux-gnu/release/websvc ./bin-gnu \
	&& cp ./bin/healthcheck ./bin-gnu

# Prevent reading+writing to the binaries, making them execute-only
RUN chmod -rw ./bin/* \
	&& chmod -rw ./bin-gnu/*

FROM alpine:latest as patched

RUN apk update && apk upgrade --no-cache

# Create a debug container with things like a shell and package manager for additional
# tools. This could be used to debug the prod binary.
# An additional candidate could be mcr.microsoft.com/cbl-mariner/distroless/debug:2.0, but
# when this was created I was getting `trivy` vuln flags for that image.
FROM patched as debug

COPY --from=builder /app/bin/ /

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/websvc"]

# Another candidate base could be mcr.microsoft.com/cbl-mariner/distroless/minimal:2.0 which
# provides filesystem, tzdata, and prebuilt-ca-certificates.
FROM scratch as prod

COPY --from=builder /app/bin/ /

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/websvc"]

# Create a debug container with things like a shell and package manager for additional
# tools. This could be used to debug the prod binary.
# An additional candidate could be mcr.microsoft.com/cbl-mariner/distroless/debug:2.0, but
# when this was created I was getting `trivy` vuln flags for that image.
FROM patched as debug-gnu

COPY --from=builder /app/bin-gnu /

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/websvc"]

# Another candidate base could be mcr.microsoft.com/cbl-mariner/distroless/minimal:2.0 which
# provides filesystem, tzdata, and prebuilt-ca-certificates.
FROM scratch as prod-gnu

COPY --from=builder /app/bin-gnu /

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/websvc"]