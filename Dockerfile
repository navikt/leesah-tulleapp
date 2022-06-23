FROM rust:1.61 as builder
RUN USER=root

RUN mkdir app
WORKDIR /app
ADD . ./
RUN cargo clean && \
    cargo build -vv --release

FROM debian:buster-slim
ARG APP=/usr/src/app
ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# Copy the compiled binaries into the new container.
COPY --from=builder /bot/target/release/leesah-tulleapp ${APP}/leesah-tuleapp
RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}

CMD ["./leesah-tulleapp"]