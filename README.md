# SPACEFLIGHTS

Foundation-first Rust + Bevy codebase for SPACEFLIGHTS.

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
- Hold `Left Mouse Button` + move mouse: look around
- `[` and `]`: change ship speed
- `1` / `2` / `3`: switch ship/module/eva mode
