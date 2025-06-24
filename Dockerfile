# This Dockerfile has been optimized for single-target compilation (AMD64)
# for faster builds on platforms like Railway's free tier.
# It removes the ARM64 cross-compilation steps.

# Builder
# Specify platform for the builder stage explicitly to ensure AMD64 toolchain
FROM --platform=linux/amd64 rust:1.80.1-slim AS builder

WORKDIR /app

# Add extra build dependencies here
# Removed gcc-aarch64-linux-gnu and binutils-aarch64-linux-gnu as we no longer cross-compile
RUN apt-get update && apt install -yqq \
    cmake curl bzip2 libpq-dev

# Removed manual ARM64 compilation of libpq as it's no longer needed
# ENV PGVER=16.4
# RUN curl -o postgresql.tar.bz2 https://ftp.postgresql.org/pub/source/v${PGVER}/postgresql-${PGVER}.tar.bz2 && \
#     tar xjf postgresql.tar.bz2 && \
#     cd postgresql-${PGVER} && \
#     ./configure --host=aarch64-linux-gnu --enable-shared --disable-static --without-readline --without-zlib --without-icu && \
#     cd src/interfaces/libpq && \
#     make

COPY . .

# Removed adding aarch64 target as we only build for x86_64
RUN rustup target add x86_64-unknown-linux-gnu

# Removed cache mount directives to bypass persistent Railway build error
RUN cargo build --release --target=x86_64-unknown-linux-gnu && \
    # Only copy the x86_64 executable
    cp /app/target/x86_64-unknown-linux-gnu/release/spoticord /app/spoticord_final_binary

# Runtime
FROM debian:bookworm-slim

# Removed TARGETPLATFORM ARG/ENV as we only have one target now
# ARG TARGETPLATFORM
# ENV TARGETPLATFORM=${TARGETPLATFORM}

# Add extra runtime dependencies here
RUN apt update && apt install -y ca-certificates libpq-dev

# Copy only the single compiled binary
COPY --from=builder \
    /app/spoticord_final_binary /usr/local/bin/spoticord

# Removed conditional copy and deletion of unused binaries
# RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ]; then \
#     cp /tmp/x86_64 /usr/local/bin/spoticord; \
#     elif [ "${TARGETPLATFORM}" = "linux/arm64" ]; then \
#     cp /tmp/aarch64 /usr/local/bin/spoticord; \
#     fi
# RUN rm -rvf /tmp/x86_64 /tmp/aarch64

EXPOSE 10000

# The ENTRYPOINT specifies the command to run when the container starts.
# It uses the 'spoticord' executable copied to /usr/local/bin.
ENTRYPOINT [ "/usr/local/bin/spoticord" ]
