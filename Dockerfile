FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

RUN apt-get update -y && apt-get install -y --no-install-recommends git openssh-client
RUN mkdir -p -m 0700 /root/.ssh && \
    ssh-keyscan github.com >> /root/.ssh/known_hosts

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=ssh cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin platform

FROM ubuntu:latest AS runtime
EXPOSE 8080

# Create a user and group 'appuser' with UID and GID 7999
RUN groupadd -g 7999 appuser && \
    useradd -r -u 7999 -g appuser appuser

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/platform platform

COPY configuration configuration
ENV APP_ENVIRONMENT=production

# Change the ownership of the /app directory to 'appuser'
RUN chown -R appuser:appuser /app

# Run the container as 'appuser'
USER appuser

ENTRYPOINT ["./platform"]