# Spaceflights MVP+ (Module Feel Prototype)

A browser-playable MVP of the `SPACEFLIGHTS` spec using pure JavaScript + HTML Canvas.
This pass intentionally focuses on one mode (`Module`) to validate gameplay feel first.

## What is implemented

- Procedural wireframe-only rendering on black background
- 3D-like depth through perspective, line thickness, and opacity fade
- Parallax starfield and infinite reference grid
- Single-mode third-person module chase camera for movement-feel testing
- Seed-based procedural world generation:
  - Same seed => same world
  - New seed => new starfield, anchors, debris, and objective layout
- Signal sphere mechanic:
  - Inside sphere: stable
  - Edge: flicker warning
  - Outside: control delay + ghosting
- Dashed trajectory prediction for module
- Smooth follow camera + center reticle + objective distance marker
- HUD resource bars and mission-state feedback
- Mission flow:
  - Reach objective
  - Perform EVA repair (`F` held near objective to build repair progress)
  - Recover module to ship
  - Mission complete
- Tension/resource systems:
  - Fuel, oxygen, signal quality, damage, heat feedback

## Controls

- `W A S D`: thrust
- `Space` / `Ctrl`: vertical thrust
- `Q` / `E`: rotate yaw
- `Shift`: boost
- `R`: reset current seed/run
- `T`: generate and load a new procedural seed

## Run

From repo root:

```bash
python3 -m http.server 8080
```

Then open:

- `http://localhost:8080/mvp/`
