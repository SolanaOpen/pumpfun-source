# Runs solopen's AI rebuild loop on the pump.fun programs. Build for linux/amd64:
# the deployed programs were built with the Linux x86-64 platform-tools, and only
# that toolchain is established to give byte-identical results.
FROM --platform=linux/amd64 python:3.12-slim-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
        git curl ca-certificates bzip2 build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
ARG SOLOPEN_REF=main
RUN git clone --depth 1 --branch "$SOLOPEN_REF" https://github.com/SolanaOpen/SolOpen /opt/solopen
WORKDIR /work
COPY run.sh checkpoint.py programs.json /opt/runner/
RUN chmod +x /opt/runner/run.sh
ENV SOLOPEN=/opt/solopen RUNNER=/opt/runner
ENTRYPOINT ["/opt/runner/run.sh"]
