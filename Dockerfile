FROM --platform=$BUILDPLATFORM ubuntu:24.04 AS build
ENV HOME="/root"
WORKDIR $HOME
SHELL [ "/bin/bash", "-o", "pipefail", "-c" ]
RUN apt-get update && apt-get install -y --no-install-recommends build-essential=* curl=* python3-venv=* clang=* lld=*
RUN python3 -m venv $HOME/.venv && .venv/bin/pip install cargo-zigbuild
ENV PATH="$HOME/.venv/bin:$PATH"
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
COPY --from=build /klip /klip
WORKDIR /
ENV HOME=/
ENTRYPOINT ["/klip"]
CMD ["serve"]
