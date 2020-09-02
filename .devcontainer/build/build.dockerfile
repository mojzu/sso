# docker pull buildpack-deps:buster
FROM buildpack-deps:buster

# <https://github.com/microsoft/vscode-dev-containers>
# <https://github.com/microsoft/vscode-dev-containers/blob/master/containers/docker-from-docker/.devcontainer/Dockerfile>
# <https://aka.ms/vscode-remote/containers/non-root-user>
ARG INSTALL_ZSH="true"
ARG UPGRADE_PACKAGES="false"
ARG ENABLE_NONROOT_DOCKER="true"
ARG SOURCE_SOCKET=/var/run/docker-host.sock
ARG TARGET_SOCKET=/var/run/docker.sock
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Install needed packages and setup non-root user. Use a separate RUN statement to add your own dependencies.
COPY .devcontainer/build/library-scripts/*.sh /tmp/library-scripts/
RUN apt-get update \
    && /bin/bash /tmp/library-scripts/common-debian.sh "${INSTALL_ZSH}" "${USERNAME}" "${USER_UID}" "${USER_GID}" "${UPGRADE_PACKAGES}" \
    # Use Docker script from script library to set things up
    && /bin/bash /tmp/library-scripts/docker-debian.sh "${ENABLE_NONROOT_DOCKER}" "${SOURCE_SOCKET}" "${TARGET_SOCKET}" "${USERNAME}" \
    # Clean up
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/* /tmp/library-scripts/

# Setting the ENTRYPOINT to docker-init.sh will configure non-root access to
# the Docker socket if "overrideCommand": false is set in devcontainer.json.
# The script will also execute CMD if you need to alter startup behaviors.
ENTRYPOINT [ "/usr/local/share/docker-init.sh" ]
CMD [ "sleep", "infinity" ]

# install_rust.sh
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.45.2

COPY .devcontainer/install_rust.sh /opt/install_rust.sh
RUN chmod +x /opt/install_rust.sh \
    && /opt/install_rust.sh

# install_node.sh
ENV NODE_VERSION=14.9.0 \
    YARN_VERSION=1.22.5

COPY .devcontainer/install_node.sh /opt/install_node.sh
RUN chmod +x /opt/install_node.sh \
    && /opt/install_node.sh

# Clean up
RUN apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=dialog

# Set cache directories in volume
ENV CARGO_HOME="/workspace/.cargo" \
    PATH=/workspace/.cargo/bin:$PATH
RUN npm config set cache /workspace/.npm --global

# Fix: Fixes docker socket has incorrect group ownership
RUN sudo chown vscode:999 /var/run/docker.sock
