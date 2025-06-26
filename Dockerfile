# Dockerfile for NAT Traversal Testing Environment
FROM rust:1.82

# install useful networking tools for debugging
RUN apt-get update && apt-get install -y \
    netcat-openbsd \
    tcpdump \
    iproute2 \
    iputils-ping \
    telnet \
    curl \
    iptables \
    && rm -rf /var/lib/apt/lists/*

# set working directory
WORKDIR /workspace

COPY . .

# build the project
RUN cargo build

# Set default command
CMD ["cargo", "run", "--bin", "signaling_server"]