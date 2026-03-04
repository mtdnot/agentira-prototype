FROM rust:1.83

# Install wasm-pack and dependencies  
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN rustup target add wasm32-unknown-unknown

# Install basic-http-server for local testing
RUN cargo install basic-http-server

# Set working directory
WORKDIR /project

# Copy source files
COPY . .

# Build WASM
RUN cargo build --target wasm32-unknown-unknown --release

# Create distribution directory
RUN mkdir -p dist && \
    cp target/wasm32-unknown-unknown/release/agentira_prototype.wasm dist/ && \
    cp static/index.html dist/

# Expose port for local testing
EXPOSE 4000

# Default command to serve the files
CMD ["basic-http-server", "dist", "-a", "0.0.0.0:4000"]