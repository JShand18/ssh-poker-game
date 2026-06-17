# Dev Container for the ssh-poker-game Go (Wish + Bubble Tea) gateway and the
# Rust engine-service. Provides Go, Rust, protoc, the Go protobuf plugins, and buf.
FROM mcr.microsoft.com/devcontainers/go:1-bookworm

# Protocol Buffers compiler.
# The base image ships a yarn apt repo whose signing key is expired, which makes
# `apt-get update` fail; drop any such repo before updating.
RUN for f in /etc/apt/sources.list.d/*; do \
        [ -f "$f" ] && grep -q yarnpkg "$f" 2>/dev/null && rm -f "$f" || true; \
    done; \
    apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends protobuf-compiler \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Rust toolchain (matches the host: 1.82.0), installed for the non-root user.
USER vscode
ENV RUSTUP_HOME=/home/vscode/.rustup \
    CARGO_HOME=/home/vscode/.cargo \
    PATH=/home/vscode/.cargo/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain 1.82.0 --profile minimal \
    && rustup component add clippy rustfmt

# Go protobuf/gRPC code generators and buf.
# GOTOOLCHAIN=auto lets `go install` fetch a newer toolchain if a tool needs one.
ENV GOTOOLCHAIN=auto
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest \
    && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest \
    && go install github.com/bufbuild/buf/cmd/buf@latest

USER root
