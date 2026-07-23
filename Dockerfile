# syntax=docker/dockerfile:1.7

FROM rust:1.95-slim-bookworm AS builder

ARG CARGO_BUILD_JOBS=4
ENV CARGO_INCREMENTAL=0 \
    CARGO_TARGET_DIR=/tmp/cargo-target

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build \
      --release \
      --locked \
      --offline \
      --bins \
      --jobs "${CARGO_BUILD_JOBS}" \
    && install -Dm755 /tmp/cargo-target/release/sender /out/sender \
    && install -Dm755 /tmp/cargo-target/release/receiver /out/receiver

FROM python:3.13-slim-bookworm AS runtime

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1

RUN groupadd --gid 10001 transport \
    && useradd --uid 10001 --gid transport --no-create-home --shell /usr/sbin/nologin transport \
    && install -d -o transport -g transport /app \
    && rm -rf /root/.cache /tmp/* /var/tmp/*

WORKDIR /app
COPY --from=builder --chown=transport:transport /out/sender /out/receiver ./
COPY --chown=transport:transport common.py endpoints.py relay.py run.py score.py ./
COPY --chown=transport:transport profiles ./profiles

USER transport
STOPSIGNAL SIGTERM

ENTRYPOINT ["python3", "run.py"]
CMD ["--profile", "profiles/B.json", "--seed", "1", "--delay_ms", "90"]
