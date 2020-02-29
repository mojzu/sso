FROM sso/build-cache:v1

# Copy project files.
ADD . /build

# Build binaries.
RUN cargo make release \
    && cargo make openapi-bin;
