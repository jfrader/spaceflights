# SPACEFLIGHTS Architecture Foundation

## Layering Rules
- `core` is engine-agnostic and contains domain contracts only.
- `app` integrates Bevy runtime, rendering, and platform concerns.
- `core` must not import Bevy.
- `features/*` communicate through shared resources/events/contracts, not direct module internals.

## Module Responsibilities
- `core::config`: app/domain-neutral configuration models.
- `core::seed`: deterministic seed model and helper generation utilities.
- `core::gameplay`: engine-agnostic gameplay state, commands, and deterministic step logic.
- `core::music`: deterministic procedural note generation contracts.
- `core::worldgen`: deterministic procedural world snapshots (starfield + horizon debris).
- `core::persistence`: engine-agnostic persistence contracts and snapshot types.
- `core::events`: shared event namespaces for cross-feature communication.
- `app::plugins`: global wiring, scheduling, and scene/bootstrap services.
- `app::persistence`: persistence strategy adapters (SQLite now, SpaceTimeDB adapter slot).
- `app::features`: feature plugins with isolated systems.
  - includes `music` feature with adapter-based synth backend abstraction.

## Scheduling Convention
- Update order is explicit and deterministic using sets:
  1. `Persistence`
  2. `Input`
  3. `Gameplay`
  4. `Music`
  5. `Camera`
  6. `World`
  7. `Ui`
  8. `Debug`
- Gameplay systems are active in current builds (`main` and `dev-debug`), with deterministic set ordering.

## Coding Conventions
- Favor small modules and explicit interfaces over large shared files.
- Avoid hidden global state; state should be in typed resources/components.
- Keep systems single-purpose and side-effect bounded.
- Add tests at boundaries (`core` deterministic logic, `app` boot/composition behavior).

## Dev Debug Build
- `spaceflights-dev-debug` runs a gameplay-only shell for fast iteration.
- It excludes persistence and full mission composition by design, but keeps gameplay + world + mouse look.
