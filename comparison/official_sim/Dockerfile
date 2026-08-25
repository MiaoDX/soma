FROM python:3.12-slim

ARG REACHY_COMMIT=b7e686d994a178353ebf81ea935de82ce65af733
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential git git-lfs pkg-config libcairo2-dev libgirepository1.0-dev \
       libgstreamer1.0-0 gir1.2-gstreamer-1.0 gir1.2-gst-plugins-base-1.0 \
    && git clone https://github.com/pollen-robotics/reachy_mini.git /opt/reachy_mini \
    && git -C /opt/reachy_mini checkout "$REACHY_COMMIT" \
    && git -C /opt/reachy_mini lfs pull \
    && python -m pip install "/opt/reachy_mini[mujoco]" \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
