# Rust 1.71.0 is latest as of July 24, 2023
FROM rust:1.71.0 as builder

# Install the targets
RUN rustup target add $(arch)-unknown-linux-musl && \
	dpkg --add-architecture $(arch) && \
	apt update && \
	apt install -y musl-tools:$(arch) musl-dev:$(arch)

WORKDIR /app

COPY . ./

# Use a statically linked target for the prod
RUN cargo build -p pixy-server --release --target $(arch)-unknown-linux-musl

RUN cargo build -p healthcheck --release --target $(arch)-unknown-linux-musl

# Coalesce all the compiled binaries into a final directory for each output
# so it's easier to copy in the next stage
RUN mkdir ./bin && \
	mv ./target/$(arch)-unknown-linux-musl/release/pixy-server \
	./target/$(arch)-unknown-linux-musl/release/healthcheck  ./bin

# Prevent reading+writing to the binaries, making them execute-only
RUN chmod -rw ./bin/* && \
	chown -R 1000:1000 ./bin

# Can use other distroless containers if desired
FROM scratch as prod

COPY --from=builder /app/bin/ /

ENV PIXY_LOG_LEVEL info
ENV PIXY_PORT 8000
ENV PIXY_CONFIG_FILE pixy.yaml

USER 1000:1000

HEALTHCHECK --interval=30s --timeout=30s --start-period=2s --retries=3 CMD [ "/healthcheck" ]

CMD ["/pixy-server"]