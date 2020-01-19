FROM sso-build:latest as build

# Copy files and build binary.
ADD . /sso/sso_openapi
RUN cargo make openapi-bin;

FROM debian:10.2
ENV DEBIAN_FRONTEND="noninteractive"

# HTTP server port.
EXPOSE 8042

# Copy and start server binary.
COPY --from=build /sso/sso_openapi/sso-openapi .

# Copy binaries.
COPY --from=build /sso/sso_openapi/sso-openapi /usr/local/bin/sso-openapi
RUN chmod +x /usr/local/bin/sso-openapi

ENTRYPOINT ["sso-openapi"]
