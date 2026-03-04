# Agentira Prototype - Moving AI

A simple 3D prototype showcasing autonomous AI agents moving around in a virtual environment, built with Rust and Macroquad.

## Features

- 🤖 **Multiple AI Agents**: 5 different colored cube agents
- 🎯 **Autonomous Movement**: Random direction changes and collision avoidance  
- 🌍 **3D Environment**: Ground plane with grid and auto-rotating camera
- 🎮 **Real-time Rendering**: Smooth 60fps 3D graphics
- 🌐 **Web Deploy**: WASM compilation for browser deployment

## Technology Stack

- **Language**: Rust 🦀
- **Game Engine**: Macroquad (lightweight 3D/2D framework)
- **Target**: WASM32 for web deployment
- **CI/CD**: GitHub Actions
- **Deployment**: GitHub Pages

## AI Agent Behavior

Each agent has:
- Independent movement speed (1.5-3.5 units/sec)
- Random direction changes (1-3 second intervals)
- Boundary collision detection and response
- Visual direction indicators

## Development

### Local Development
```bash
cargo run
```

### WASM Build
```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### Deploy
Push to `main` branch - GitHub Actions will automatically:
1. Build WASM binary
2. Create web assets  
3. Deploy to GitHub Pages

## Architecture

This prototype demonstrates the feasibility of:
- **AI-first game development**: All logic in code, no visual editors
- **Lightweight 3D**: Minimal dependencies for fast iteration
- **Web deployment**: No installation required for testing

## Next Steps

1. ✅ **Basic 3D + AI movement** (current)
2. 🔲 **Resource gathering mechanics**
3. 🔲 **Player instruction system**  
4. 🔲 **Multi-agent coordination**
5. 🔲 **Factory building elements**

## Part of Agentira Project

This is a technical feasibility test for the larger Agentira educational game - an AI agent management game where players learn prompt engineering and automation thinking through factory building mechanics.

---

**Status**: ✅ Prototype complete, ready for deployment testing