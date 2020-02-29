FROM sso/build-cache:v1

# Copy project files.
ADD ./sso /build/sso
ADD ./sso_openapi /build/sso_openapi
ADD ./Makefile.toml /build/Makefile.toml

# Build binaries.
RUN cargo make release \
    && cargo make openapi-bin;
