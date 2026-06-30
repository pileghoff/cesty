ARG LLVM_VERSION=18

FROM rust:1.90-slim-bookworm

ARG LLVM_VERSION

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget \
        gnupg \
        ca-certificates && \
    wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | \
        gpg --dearmor -o /usr/share/keyrings/llvm.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/llvm.gpg] http://apt.llvm.org/bookworm/ llvm-toolchain-bookworm-${LLVM_VERSION} main" \
        > /etc/apt/sources.list.d/llvm.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
        clang-${LLVM_VERSION} \
        lld-${LLVM_VERSION} \
        llvm-${LLVM_VERSION}-dev && \
    rm -rf /var/lib/apt/lists/*

ENV CC=clang-${LLVM_VERSION}
ENV CXX=clang++-${LLVM_VERSION}
