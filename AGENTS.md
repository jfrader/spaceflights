## Agent Instructions

- Read relevant official library docs before changing behavior.
- Fix root cause, not symptoms: reproduce, trace, verify against SPEC.
- Preserve architecture on every change: deterministic/domain logic in `core`, Bevy/runtime in `app`; keep systems small/composable and update tests/docs/changelog for significant changes.
