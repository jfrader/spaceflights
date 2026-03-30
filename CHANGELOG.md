# Changelog

All significant changes must be recorded here.

## Rules
- Update this file in the same PR/commit as the significant change.
- Use short Conventional Commit-style messages.
- Keep entries focused on user-visible or architecture-significant changes.
- Ignore trivial refactors, formatting-only edits, and typo-only changes.

## Message format
- `type(scope): short summary`

## Allowed types
- `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `build`, `ci`, `chore`

## Unreleased
- `ci(build): install ALSA + udev dev packages on Linux runners so rodio/alsa-sys and libudev-sys builds pass in clippy/tests/releases`
- `ci(release): add cross-platform release workflow (linux/macos/windows) with build artifacts and tag-based GitHub Release assets`
- `fix(world): replace thin star shell with radial volume distribution to remove universe-edge ring artifacts`
- `docs(architecture): align schedule docs with camera-before-world ordering and active gameplay phase`
- `fix(world): prevent starfield shell-outside artifact at high mission distance and randomize star depth to remove universe-edge ring`
- `fix(world): stabilize seeded starfield with pixel-calibrated star scale and FXAA camera pass to reduce shimmer/flicker`
- `fix(world): anchor star shell to ship-space center with fixed seeded radius to eliminate camera-orbit star flicker`
- `fix(camera): enforce deterministic camera flow order (mouse look before follow camera) to remove frame-order star jitter`
- `fix(render): disable 3D tonemapping/deband dither for stable star brightness during camera motion`
- `fix(world): resolve starfield jitter by sampling camera Transform in-frame and running camera systems before world sync`
- `refactor(debug): schedule dev follow-camera in GameSet::Camera for deterministic render ordering`
- `fix(world): generate seeded uniform sky directions for stars to prevent camera-angle dropout`
- `feat(worldgen): increase base star counts in main/dev profiles for denser space readability`
- `fix(world): tune planet annulus width thinner and raise star size/brightness for closer MVP visual parity`
- `refactor(world): replace segmented planet rim with single annulus + disk meshes to remove close-range border dropout`
- `fix(world): decouple star shell behavior from camera orbit and base it on ship-planet distance for stable look-around`
- `fix(world): render stars as small spheres instead of cube points to reduce camera-motion shimmer`
- `fix(world): stabilize star visibility by scaling point size with background shell distance`
- `fix(world): increase star shell planet margin and widen flyby spline clearance to avoid close-pass artifacts`
- `fix(world): clamp background star shell radius so stars remain visible during long-distance flyby departure`
- `fix(world): keep planet rim visible at close range by pushing ring segments outside occluder depth overlap`
- `fix(world): widen dev flyby spline to avoid overly close planet passes`
- `fix(render): improve line stability with explicit MSAA and corrected camera near/far planes`
- `fix(world): keep seeded star shell behind planets by deriving shell radius from camera-to-planet distance`
- `fix(world): render stars as seeded background shell so planets stay foreground silhouettes without star bleed-through`
- `fix(world): add opaque planet occluder and curved one-shot dev flyby trajectory (no straight-through crossing)`
- `feat(world): gate flyby demo behavior to dev-debug build while preserving seeded worldgen in main build`
- `fix(world): restore visible motion by rescaling world render space and retuning star/debris parallax`
- `fix(world): stabilize star flicker by slowing parallax and correct one-shot planet flyby direction`
- `fix(world): remove residual solid flyby disks by forcing streak-only flyby rendering`
- `fix(world): make planet flyby one-shot (no repeat cycle) with continuous post-pass drift`
- `fix(world): enforce outline-only planet border and reduce round flyby blobs`
- `fix(world): replace dotted planet ring with continuous border mesh and speed up dev flyby pace`
- `fix(world): reduce debris density/size and anchor flyby debris to ship-local motion`
- `fix(world): rebalance particle sizes (smaller debris/flyby/planet points, brighter/larger stars)`
- `fix(world): reduce star/planet shimmer by removing additive flicker and camera-facing ring jitter`
- `fix(world): render planet as single wireframe-style circumference and tune flyby approach`
- `feat(debug): randomize initial seed on every dev debug launch`
- `feat(world): add particle-based planet anchor with orbital ring for flyby/orbit reference`
- `refactor(world): switch space visuals to additive particle-style points and streaks`
- `feat(world): switch stars/debris/flybys to sphere-based curved visuals (no boxy primitives)`
- `feat(world): add near-field recycled flyby layer for visible comet/asteroid passers`
- `feat(world): differentiate stars and debris with distinct geometry, scale, and color palettes`
- `feat(audio): render layered lo-fi procedural phrases (melody, bass, pad, drums, texture)`
- `fix(build): make run targets explicit and default runtime music backend to rodio`
- `feat(audio): add rodio procedural synth backend for audible runtime music`
- `feat(core): add deterministic procedural music generation module`
- `feat(app): add adapter-based procedural synth framework with configurable backends`
- `feat(app): wire music systems into main and dev debug build schedules`
- `docs(readme): add procedural music configuration variables`
- `feat(core): add deterministic worldgen module for starfield and horizon debris`
- `feat(app): replace world scaffold with seeded star/debris rendering and bounded streaming`
- `feat(camera): add hold-click mouse look with configurable sensitivity and pitch clamp`
- `feat(debug): enable procedural world and mouse look in gameplay dev debug build`
- `test(world): add integration coverage for world initialization and bounded entity counts`
- `feat(app): add gameplay-only dev debug build with dedicated binary target`
- `test(app): add headless integration coverage for dev debug build boot`
- `feat(core): add deterministic gameplay state machine with command-based step function`
- `feat(app): add gameplay feature plugin with fixed-timestep simulation loop`
- `feat(input): map keyboard controls to gameplay commands and mode switching`
- `feat(ui): sync overlay model with gameplay runtime and persistence backend`
- `feat(core): bootstrap Rust workspace with core/app crate boundaries`
- `feat(app): add Bevy plugin/schedule foundation with deterministic system sets`
- `feat(config): add env-driven app configuration with deterministic seed contract`
- `feat(persistence): add strategy-based gameplay persistence abstraction`
- `feat(persistence): implement SQLite adapter and SpaceTimeDB adapter stub`
- `test(app): add integration coverage for plugin boot, schedule order, and backend selection`
- `docs(architecture): document layering rules and persistence scheduling`
- `build(ci): add fmt/clippy/test workflow and strict linting defaults`
