FROM rust:alpine3.17 AS builder


# Set the environment variables for pkg-config
ENV export PKG_CONFIG_SYSROOT_DIR=/usr/lib
ENV export PKG_CONFIG_LIBDIR=/usr/lib/pkgconfig

# Instale as dependências necessárias, incluindo o pacote libstdc++.
RUN apk add --no-cache g++ libstdc++ musl-dev zeromq-dev pkgconfig

WORKDIR /app

COPY ./ .

# Adicione a configuração ao arquivo cargo/config.toml
RUN mkdir -p .cargo && \
    echo '[target.x86_64-unknown-linux-musl]' >> .cargo/config.toml && \
    echo 'rustflags = ["-C", "target-feature=-crt-static"]' >> .cargo/config.toml

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

RUN chmod +x start.sh


####################################################################################################
## Final image
####################################################################################################
FROM alpine:3.17


WORKDIR /code

# Install runtime dependencies for ZeroMQ
RUN apk add --no-cache libzmq

# Copy our build
COPY --from=builder /app/start.sh /usr/local/bin/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/book /usr/local/bin/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/tick /usr/local/bin/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/full /usr/local/bin/


ENTRYPOINT ["/bin/sh", "/usr/local/bin/start.sh"]
