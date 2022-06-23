FROM debian:buster-slim
COPY ./leesah-tulleapp /app/
ENV RUST_LOG=info
ENTRYPOINT ["/app/leesah-tulleapp"]