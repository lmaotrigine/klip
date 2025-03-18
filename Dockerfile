FROM --platform=$BUILDPLATFORM ubuntu:24.04 AS build
ENV HOME="/root"
WORKDIR $HOME
SHELL [ "/bin/bash", "-euo", "pipefail", "-c" ]
RUN apt-get update && apt-get install -y --no-install-recommends \
  git=* build-essential=* curl=* python3-venv=* clang=* lld=*
RUN python3 -m venv $HOME/.venv && .venv/bin/pip install cargo-zigbuild
ENV PATH="$HOME/.venv/bin:$PATH"
RUN printf '#!/bin/sh\n/root/.venv/bin/python -m ziglang "$@"\n' > /usr/bin/zig && \
  chmod +x /usr/bin/zig
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
  "linux/arm64") echo "aarch64-unknown-linux-musl" > rust_target.txt ;; \
  "linux/amd64") echo "x86_64-unknown-linux-musl" > rust_target.txt ;; \
  *) exit 1 ;; \
  esac
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
  sh -s -- -y --target "$(cat rust_target.txt)" --profile minimal --default-toolchain none
ENV PATH="$HOME/.cargo/bin:$PATH"
COPY rust-toolchain.toml rust-toolchain.toml
RUN rustup target add "$(cat rust_target.txt)"
COPY . .
RUN --mount=type=cache,target=/root/.cargo/registry \
  --mount=type=cache,target=/root/.cargo/git \
  --mount=type=cache,target=/root/target \
  cargo zigbuild --target "$(cat rust_target.txt)" --release --locked && \
  cp "target/$(cat rust_target.txt)/release/klip" /klip

FROM scratch
LABEL org.opencontainers.image.source="https://github.com/lmaotrigine/klip"
LABEL org.opencontainers.image.authors="isis@5ht2.me"
LABEL org.opencontainers.image.title="klip"
LABEL org.opencontainers.image.description="Copy/paste anything over the network."
LABEL org.opencontainers.image.licenses="MPL-2.0"
COPY --from=build /klip /klip
WORKDIR /
ENV HOME=/
RUN ["/klip", "--version"]
ENTRYPOINT ["/klip"]
CMD ["serve"]
