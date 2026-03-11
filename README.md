# Star Wars Rust Battle

A 1v1 local multiplayer space combat game built with Rust and [ggez](https://ggez.rs/).

Two players share one keyboard and fight across a split screen. The Imperial ship controls the left half; the Rebel ship controls the right half. Each player starts with **10 HP**. First to drain the opponent's health to 0 wins.

---

## How to Run

```bash
cargo run           # debug build
cargo run --release # recommended for gameplay (smoother)
```

Requires Rust + Cargo. All assets are bundled in `./assets/`.

---

## Controls

| Action | Imperial (left) | Rebel (right) |
|--------|----------------|---------------|
| Move up | `W` | `↑` |
| Move down | `S` | `↓` |
| Move left | `A` | `←` |
| Move right | `D` | `→` |
| Fire laser | `Left Shift` | `Right Alt` |

- Each player can have at most **3 active bullets** on screen at a time.
- Ships cannot cross the center dividing line.
- The game ends when either ship reaches **0 HP** — a GAME OVER screen announces the winner.

---

## Gameplay Tips

- Fire early and dodge — bullets travel fast and there's no respawn.
- You can't spam: fire a shot, dodge, fire again.
- Stay near the center line to keep pressure on your opponent.
