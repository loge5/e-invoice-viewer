# Build environment of the AppImage (spec 0020, ADR 0004): Ubuntu 24.04 with
# packages from a fixed snapshot date, the Rust toolchain of `rust-version`, and
# linuxdeploy with its GTK plugin, all fixed in version and checksum.
FROM ubuntu:24.04@sha256:73b999393d07f2bf65a40373778a44dc9a6d54ad7718d29247d8ff9413c4d6a8

ARG SNAPSHOT
ARG RUST_VERSION
ENV DEBIAN_FRONTEND=noninteractive

# The snapshot server needs TLS, so ca-certificates comes first from the
# image's own package lists, then everything is taken from the snapshot.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && echo "APT::Snapshot \"$SNAPSHOT\";" >/etc/apt/apt.conf.d/50snapshot \
    && apt-get update \
    && apt-get install -y --no-install-recommends --allow-downgrades \
        ca-certificates curl file gettext build-essential pkg-config \
        libgtk-4-dev libadwaita-1-dev libpoppler-glib-dev libgtksourceview-5-dev \
        libxml2-dev libclang-dev \
        librsvg2-common adwaita-icon-theme desktop-file-utils \
    && rm -rf /var/lib/apt/lists/*

COPY fetch.sh /opt/fetch.sh

ENV RUSTUP_HOME=/opt/rustup RUSTUP_TOOLCHAIN=$RUST_VERSION PATH=/opt/cargo/bin:$PATH
RUN . /opt/fetch.sh \
    && fetch rustup \
        https://static.rust-lang.org/rustup/archive/1.29.3/x86_64-unknown-linux-gnu/rustup-init \
        b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55 /tmp/rustup-init \
    && chmod +x /tmp/rustup-init \
    && CARGO_HOME=/opt/cargo /tmp/rustup-init -y --no-modify-path --profile minimal \
        --default-toolchain "$RUST_VERSION" \
    && rm /tmp/rustup-init \
    && chmod -R a+rX /opt/rustup /opt/cargo

RUN . /opt/fetch.sh \
    && fetch linuxdeploy \
        https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage \
        c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d /opt/linuxdeploy \
    && fetch linuxdeploy-plugin-gtk \
        https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/7a3fbc31a9e5075073ff8790f26effbac5f84453/linuxdeploy-plugin-gtk.sh \
        b0f4cbc684a0103a9651f0955b635eaea0096b3a66c0f5a2c2aa337960375171 /opt/linuxdeploy-plugin-gtk.sh \
    && chmod 755 /opt/linuxdeploy /opt/linuxdeploy-plugin-gtk.sh
