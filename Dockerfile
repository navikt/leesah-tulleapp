FROM gcr.io/distroless/static:nonroot
COPY --chown=nonroot:nonroot ./leesah-tulleapp /app/
EXPOSE 8999
ENV RUST_LOG=info
ENTRYPOINT ["/app/leesah-tulleapp"]