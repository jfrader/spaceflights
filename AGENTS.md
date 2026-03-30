## Agent Instructions

### Doc Priority
- `AGENTS.md` -> `README.md` (current slice) -> `docs/ARCHITECTURE.md` -> `docs/SPEC.md`.
- If docs conflict, do not guess: align docs in the same change or ask.
- Read relevant official library docs before changing behavior.
- Fix root cause, not symptoms: reproduce, trace, verify against SPEC.
- Preserve architecture on every change: deterministic/domain logic in `core`, Bevy/runtime in `app`; keep systems small/composable and update tests/docs/changelog for significant changes.
- Avoid hardcoded behavior: model volatile rules as typed config/contracts and adapt them at module boundaries.
