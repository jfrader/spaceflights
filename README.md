# SPACEFLIGHTS

[![ci](https://img.shields.io/github/actions/workflow/status/jfrader/spaceflights/ci.yml?branch=master&style=flat&label=ci)](https://github.com/jfrader/spaceflights/actions)
[![license](https://img.shields.io/github/license/jfrader/spaceflights?style=flat)](./LICENSE)
[![last commit](https://img.shields.io/github/last-commit/jfrader/spaceflights?style=flat)](https://github.com/jfrader/spaceflights/commits)

Foundation-first Rust + Bevy codebase for SPACEFLIGHTS.

## Current Slice (Agent Checklist)
- Third-person gameplay feel first (not full mission depth yet).
- Space must read as deep: star background acts as distant orientation reference.
- Dev demo (`run-dev`) includes a nearby planet flyby, then departure to a new trajectory.
- Visual baseline matches spec intent: black background, sparse wireframe-like readability, seeded procedural world.

## Current Implementation Notes
- `run-dev` is the visual/gameplay-feel sandbox; `run` remains more conservative.
- Planet silhouette uses an internal black occluder + outline to preserve readability against stars.
- Full mission loop depth is intentionally deferred while feel/architecture are stabilized.

## Project Layout
- `app/` - Bevy runtime app and plugin composition
- `core/` - engine-agnostic domain contracts
- `docs/` - project documentation and specs
- `mvp/` - legacy prototype (reference only)

## Docs
- `docs/SPEC.md`
- `docs/ARCHITECTURE.md`
- `docs/IDEA.md`
- `CHANGELOG.md` (must be updated for significant changes)

## Common Commands
- `make run` (main build, defaults to `SPACEFLIGHTS_MUSIC_BACKEND=rodio`)
- `make run-dev` (gameplay-only developer debug build, defaults to `rodio` and a fresh seed every launch)
- `make check`
- `make test`

## Release Builds (GitHub Actions)
- Workflow: `.github/workflows/release.yml`
- Triggers:
  - Manual: Actions -> `release-builds` -> `Run workflow`
  - Tag push: `git tag v0.1.0 && git push origin v0.1.0`
- Output:
  - Per-platform artifacts (`linux-x64`, `macos-arm64`, `windows-x64`) containing:
    - `spaceflights-app`
    - `spaceflights-dev-debug`
    - `README.md`
    - `QUICKSTART.txt`
  - On `v*` tags, zipped artifacts are also attached to a GitHub Release.

Override music backend per run:
- `make run MUSIC_BACKEND=silent`
- `make run-dev MUSIC_BACKEND=debug`

## Persistence Backend
- Default backend: SQLite (`:memory:` in foundation mode)
- Switch backend via env:
  - `SPACEFLIGHTS_PERSISTENCE_BACKEND=sqlite`
  - `SPACEFLIGHTS_PERSISTENCE_BACKEND=spacetimedb`

## World + Camera Tuning
- `SPACEFLIGHTS_WORLD_PROFILE=main|dev_debug`
- `SPACEFLIGHTS_WORLD_STAR_DENSITY=100`
- `SPACEFLIGHTS_WORLD_DEBRIS_DENSITY=100`
- `SPACEFLIGHTS_MOUSE_SENSITIVITY=0.0022`
- `SPACEFLIGHTS_MOUSE_PITCH_LIMIT_DEG=80`

Notes:
- Planet flyby motion is intentionally enabled in `run-dev` only (dev demo pass).
- Main `run` keeps seeded procedural world visuals without dev flyby choreography.

## Procedural Music
- `SPACEFLIGHTS_MUSIC_ENABLED=true|false`
- `SPACEFLIGHTS_MUSIC_BACKEND=silent|debug|rodio`
- `SPACEFLIGHTS_MUSIC_BPM=104`
- `SPACEFLIGHTS_MUSIC_SCALE=minor|major`

## Controls
- Controls are action-bound via config (not hardcoded), with defaults below.
- `Enter`: start from main menu
- `Esc`: open main menu while in-game
- `N`: start a new random seeded world
- `R`: restart current seed
- Hold `Left Mouse Button` + move mouse: look around
- `W` / `S`: accelerate / brake
- `1` / `2` / `3`: switch ship/module/eva mode
- `A` / `D`: steer yaw left/right
- Arrow `Up` / `Down`: pitch up/down

UI flow:
- Main menu on startup
- Lightweight HUD while flying (mode/speed/seed/star count)

Control binding env vars:
- `SPACEFLIGHTS_KEY_SPEED_UP`
- `SPACEFLIGHTS_KEY_SPEED_DOWN`
- `SPACEFLIGHTS_KEY_MODE_SHIP`
- `SPACEFLIGHTS_KEY_MODE_MODULE`
- `SPACEFLIGHTS_KEY_MODE_EVA`
- `SPACEFLIGHTS_KEY_YAW_LEFT`
- `SPACEFLIGHTS_KEY_YAW_RIGHT`
- `SPACEFLIGHTS_KEY_PITCH_UP`
- `SPACEFLIGHTS_KEY_PITCH_DOWN`
- `SPACEFLIGHTS_KEY_MENU_TOGGLE`
- `SPACEFLIGHTS_KEY_MENU_CONFIRM`
- `SPACEFLIGHTS_KEY_MENU_NEW_WORLD`
- `SPACEFLIGHTS_KEY_MENU_RESTART_WORLD`
