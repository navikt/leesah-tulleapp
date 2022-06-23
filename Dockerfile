FROM rust:1.61-slim-buster as builder
RUN USER=root

RUN mkdir app
WORKDIR /app
ADD . ./
RUN apt-get update -y
RUN apt-get install -y \
    cmake g++ gcc libssl-dev openssl
RUN cargo clean && \
    cargo build -vv --release

FROM debian:buster-slim
ARG APP=/usr/src/app
ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# Copy the compiled binaries into the new container.
COPY --from=builder /app/target/release/leesah-tulleapp ${APP}/leesah-tulleapp
RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}

CMD ["./leesah-tulleapp"]