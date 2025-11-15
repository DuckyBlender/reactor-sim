# reactor-sim

A small reactor control game built with [Bevy](https://bevyengine.org/). Your goal is to run a nuclear power plant as efficiently as possible without melting the reactor core or destroying the turbine.

## Gameplay

- Adjust **reactivity** to heat up the reactor and produce steam.
- Adjust the **turbine** flow to turn heat into power and money.
- Keep an eye on:
  - **Reactor temperature & pressure**
  - **Turbine temperature & durability**
  - **Generated power & money**
- If reactor or turbine temperature exceeds the safety limit, the game ends.

### Uranek (operator assistant)

In endless mode the operator assistant **Uranek** sits in the top‑left corner:

- Blinks idly while you play.
- Shows a speech bubble and plays talking sounds when:
  - Reactor is close to overheating.
  - Turbine is close to overheating.
  - You are earning a lot of money.
- His greeting is visible for a short time at the start of a run.

## Controls

Keyboard:

- `ESC` – Pause / resume game.
- In menus: use mouse to navigate and click buttons.

UI:

- Use the **sliders** to set target reactivity and turbine flow.
- Watch the **gauges** on the left to keep all values in safe ranges.

## Building and running

Requirements:

- Rust (stable) and Cargo installed.
- A system that can run Bevy (Linux, macOS, or Windows with graphics drivers).

Clone and run:

```bash
cargo run --release
```

Assets are located in the `assets/` directory and loaded at runtime (fonts, sprites, models, and sounds). Make sure you run the binary from the project root so relative asset paths resolve correctly.

## Project structure

- `src/`
  - `main.rs` – Game entry point, Bevy app setup.
  - `simulation.rs` – Reactor, turbine, and environment simulation logic.
  - `ui/` – 2D UI (gauges, sliders, Uranek, menus).
  - `sound.rs` – Background music and audio settings.
  - `menu.rs`, `tutorial.rs`, `model.rs` – Menus, tutorial flow, and 3D reactor model.
- `assets/`
  - `fonts/` – UI fonts.
  - `sprites/` – 2D sprites (Uranek, UI art).
  - `sound/` – Background music, SFX and Uranek talking clips.
  - `models/` – 3D reactor model.

## License

This project is for hackathon / learning purposes. Adapt or extend it as needed.
