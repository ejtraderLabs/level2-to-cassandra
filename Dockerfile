##################################################################################################
# Builder
###################################################################################################
FROM rust:latest AS builder


RUN apt update && apt install -y libzmq3-dev -y && apt install libcomerr2 -y
RUN update-ca-certificates

RUN /sbin/ldconfig -v
WORKDIR /app

COPY ./ .

RUN cargo build --release

# FROM bitcoinnanolabs/rust AS builder

####################################################################################################
## Final image
####################################################################################################
FROM gcr.io/distroless/cc


WORKDIR /app
ENV ENVIRONMENT full

# Copy our build
COPY --from=builder /app/target/release/book ./
COPY --from=builder /app/target/release/tick ./
COPY --from=builder /app/target/release/full ./
COPY --from=builder /usr/lib/x86_64-linux-gnu/libzmq* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libbsd* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libsodium* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libpgm* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libnorm* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgssapi* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libmd* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libkrb5* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libk5crypto* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcomo* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/libkeyutils* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libkeyutils* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libcom_err* /lib/x86_64-linux-gnu/



CMD ["/app/${ENVIRONMENT}"]

