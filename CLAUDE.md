# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo run            # Build and run the game (debug)
cargo run --release  # Build and run optimized
cargo build          # Build only
cargo check          # Check for compile errors without building
cargo test           # Run tests
```

## Architecture

This is a **single-file 2D arcade game** (`src/main.rs`, ~349 lines) using the **ggez 0.9.3** game engine.

### Game Loop

Implemented via ggez's `EventHandler` trait on `GameState`:
- `update()` — Frame logic at 60 FPS (bullet movement, collision detection, win condition)
- `draw()` — Renders background, ships, bullets, health text, and win message
- `key_down_event()` — Handles keyboard input for both players

### Core Structs

| Struct | Role |
|--------|------|
| `GameState` | Top-level container: ships, bullet vecs, loaded images/audio, `game_over` flag |
| `Spaceship` | Either player's ship — holds `x, y, width, height, health`; has `intersects()` for AABB collision |
| `Bullet` | Projectile with `x, y, width, height` |

### Game Logic Flow

1. **Startup** (`main()`): Load assets from `./assets/`, create both ships, init `GameState`, start event loop.
2. **Each frame** (`update()`): Move bullets, check collisions (decrement health, play sound, remove bullet), check win condition.
3. **Collision**: `update_rebel_bullets()` / `update_imperial_bullets()` handle movement direction and hit detection against the opposing ship.

### Controls

| Player | Move | Fire |
|--------|------|------|
| Imperial (left) | WASD | Left Shift |
| Rebel (right) | Arrow keys | Right Alt |

A vertical center line divides the play area — ships cannot cross it.

### Key Constants

```rust
WINDOW_WIDTH: 1600.0  WINDOW_HEIGHT: 900.0
FPS: 60               MAX_BULLETS: 3 (per player)
BULLET_VEL: 25.0      // pixels per frame
```

### Assets

All assets live in `./assets/` and are loaded at startup. The game will panic if any asset is missing.

- `bg_version_2.jpg` — background
- `rebel_spaceship.png` / `imperial_spaceship.png` — ship sprites
- `laser.mp3` / `explosion.mp3` — sound effects
