FROM library/debian:buster AS builder

RUN apt-get update

# Rust Compiler
RUN apt-get install -y curl build-essential
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH

# Copy project into docker container ...
WORKDIR /build/telemetry-http-service
COPY telemetry-http-service .
COPY rainguage-messages ../rainguage-messages
RUN cargo +nightly build --release

# Runtime
FROM library/debian:buster
LABEL author="Michael Fletcher <m.fletcher@theplanet.ca>"
USER root

COPY --from=builder /build/telemetry-http-service/target/release/telemetry-http-service /opt/telemetry-http-service

RUN chmod a+rwx /opt/telemetry-http-service
ENTRYPOINT ["/opt/telemetry-http-service"]