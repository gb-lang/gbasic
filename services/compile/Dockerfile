# Compile-service container image.
#
# Builds the gbasic compiler + this service in one stage, then ships
# a slim runtime image with both binaries plus the LLVM/wasm-ld/wasm-opt
# tooling that `gbasic --target web` shells out to.

FROM rust:1.83-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    gnupg \
    wget \
    && wget -qO /etc/apt/trusted.gpg.d/apt.llvm.org.asc https://apt.llvm.org/llvm-snapshot.gpg.key \
    && echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main" > /etc/apt/sources.list.d/llvm.list \
    && rm -rf /var/lib/apt/lists/*

# LLVM 18 (needed by inkwell) + wasm-ld + wasm-opt + cmake (for SDL2 build,
# even though desktop runtime is unused at runtime, the compiler crate
# pulls in workspace deps).
RUN apt-get update && apt-get install -y --no-install-recommends \
    llvm-18-dev \
    libpolly-18-dev \
    lld-18 \
    binaryen \
    cmake \
    pkg-config \
    libssl-dev \
    libzstd-dev \
    && rm -rf /var/lib/apt/lists/*

ENV LLVM_SYS_180_PREFIX=/usr/lib/llvm-18

WORKDIR /build
COPY . .

RUN cargo build --release -p gbasic \
 && cargo build --release -p gbasic-compile-service

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    gnupg \
    wget \
    && wget -qO /etc/apt/trusted.gpg.d/apt.llvm.org.asc https://apt.llvm.org/llvm-snapshot.gpg.key \
    && echo "deb http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-18 main" > /etc/apt/sources.list.d/llvm.list \
    && rm -rf /var/lib/apt/lists/*

RUN apt-get update && apt-get install -y --no-install-recommends \
    llvm-18 \
    lld-18 \
    binaryen \
    libgcc-s1 \
    libstdc++6 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Make wasm-ld and wasm-opt discoverable under their canonical names.
RUN ln -sf /usr/bin/wasm-ld-18 /usr/local/bin/wasm-ld

RUN useradd --system --create-home --uid 1001 gbasic
USER gbasic
WORKDIR /home/gbasic

COPY --from=builder /build/target/release/gbasic        /usr/local/bin/gbasic
COPY --from=builder /build/target/release/gbasic-compile-service /usr/local/bin/gbasic-compile-service

ENV PORT=8080
ENV GBASIC_BIN=/usr/local/bin/gbasic
ENV RUST_LOG=info

EXPOSE 8080

# tini is recommended on Fly.io but not required; Cloudflare Containers handles signals.
CMD ["/usr/local/bin/gbasic-compile-service"]
