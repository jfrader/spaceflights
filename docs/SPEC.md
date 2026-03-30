# GAME DESIGN SPEC — "WORKING TITLE" (UPDATED)

---

## 1. Core Concept

A minimalist space exploration game where the player commands a small autonomous vessel, deploys a precision-controlled exploration module, and occasionally exits as an astronaut to perform high-risk tasks in a 3D wireframe universe.

The game presents **3D space and physics through a procedural wireframe visual system**, creating depth, scale, and motion using only lines, points, and simple geometry.

The experience blends:

* System management (inside the ship)
* Precision navigation (module control)
* Vulnerable physical interaction (astronaut)

---

## 2. Design Pillars

### 2.1 Procedural Wireframe Aesthetic

* All objects rendered as lines
* No textures, no materials
* Minimal or no filled geometry
* Entirely generatable via code
* Strong readability through abstraction

---

### 2.2 Multi-Layered Gameplay

* Ship (strategy / systems)
* Module (precision control)
* Astronaut (risk / interaction)

---

### 2.3 Guided Physics (Not Simulation)

* Inertia-inspired but assisted movement
* Player intent prioritized over realism
* Smooth, controllable navigation

---

### 2.4 Structured Mission Flow

* Earth → Station → Mission Site → Return
* Each phase has distinct pacing

---

### 2.5 Grounded Tension

* No enemies or combat
* Tension from:

  * distance
  * limited resources
  * signal loss
  * precision requirements

---

## 3. Player Role

The player controls:

* Main ship (systems + oversight)
* Exploration module (direct control)
* Astronaut (manual high-risk tasks)

Core tension:

* Attention management
* Resource balancing
* Spatial awareness

---

## 4. Core Gameplay Loop

1. Start at Earth / station
2. Receive mission
3. Travel (autopilot)
4. Dock / prepare
5. Travel to target
6. Configure systems
7. Deploy module
8. Complete objective
9. EVA if needed
10. Handle complications
11. Recover module
12. Return
13. Upgrade

---

## 5. Game Modes

### 5.1 Ship (Command Mode)

* Indirect control
* System management

Systems:

* Power distribution
* Fuel
* Signal
* Heat / damage

---

### 5.2 Module (Pilot Mode)

* Direct control
* Precision navigation

Controls:

* Thrust
* Rotation
* Boost

Constraints:

* Fuel
* Signal range
* Fragility

---

### 5.3 Astronaut (EVA)

* Manual control
* High risk

Features:

* Tether system
* Limited oxygen
* Fine movement

---

## 6. Mode Switching

* Instant switching
* Real-time continuity

---

## 7. World Structure

### Locations

* Earth
* Stations
* Planets / moons

### Environment

* Large-scale
* Sparse but readable
* Geometric clarity

### 7.1 Real-Scale Feeling

The universe must feel physically large, not toy-sized.

Requirements:

* Celestial bodies (star/planets/moons) should read as massive bodies
* Bodies should usually start far from the player ship
* Travel should communicate long-distance movement over time
* Relative velocity inheritance must be preserved when detaching module/EVA from moving ship
* Scale must be felt through motion/parallax, not just UI numbers

Practical interpretation for implementation:

* Keep world distances large enough that approach takes sustained travel
* Keep nearby gameplay still navigable (do not make mission targets unreachable)
* Preserve orientation cues while traveling at high cruise speeds

---

## 8. VISUAL & RENDERING SYSTEM (CRITICAL)

This section defines EXACTLY how the game looks and must be implemented.

---

### 8.1 Rendering Constraints

Everything must be generated from:

* Line segments
* Point clouds
* Low-poly meshes (edges only)

NOT allowed:

* Textures
* Complex shaders
* High poly models

---

### 8.2 Base Rendering Rules

Background:

* Pure black

Lines:

* White by default
* Variable thickness
* Distance-based opacity

Color accents:

* Red = danger
* Blue = objective
* Green = safe

---

### 8.3 Depth Representation

Depth is achieved using:

* Line thickness (near = thicker)
* Opacity fade (far = dim)
* Occlusion (hidden lines not drawn)

---

### 8.4 Spatial Readability & Orientation (NEW REQUIREMENT)

Space must NEVER feel visually empty or disorienting.

Rules:

* The player must always have orientation references on screen
* There must always be visible distant reference points
* The player must always be able to tell direction and movement

Systems that ensure this:

Starfield:

* Always visible
* Multiple parallax layers
* Provides motion reference

Distant Anchors:

* Planets, stations, or large structures always visible at long distances
* Never fully culled from view

Reference Grid:

* Faint 3D grid provides spatial grounding

Result:

* Player always understands where they are and where they are looking

---

### 8.5 Starfield (Parallax)

* Multiple layers of points
* Different movement speeds

Purpose:

* Motion feedback
* Depth
* Orientation reference (critical)

---

### 8.6 Infinite Reference Grid

* Faint 3D grid
* Aligned to world axes
* Fades with distance

Purpose:

* Show drift and velocity

---

### 8.7 Signal Sphere (Core Mechanic)

* Wireframe sphere around ship

Behavior:

* Inside: stable
* Edge: flicker
* Outside: distortion + delay

---

### 8.8 Trajectory Prediction

* Dashed line from module
* Shows future motion

---

### 8.9 Tether System (Visual)

* Line between astronaut and anchor

States:

* Relaxed: straight
* Tension: stretched + red
* Stress: vibration

---

### 8.10 Terrain

* Very low-poly meshes
* Large triangles only
* Wireframe only

---

### 8.11 Object Hierarchy

* Stars: points
* Debris: triangles
* Planets/moons (distant anchors): circumference-only wireframe silhouette (outline ring), no filled disk/body
* Module: simple
* Ship: medium
* Station: complex

---

### 8.12 Energy Flow (Ship)

* Lines between systems
* Thickness = power
* Animated flow

---

### 8.13 Motion Feedback

Thrusters:

* Lines extending from engines

Boost:

* Longer lines + longer trajectory

---

### 8.14 Failure Visualization

NO heavy UI — use visuals:

* Low fuel → weak thrust lines
* Heat → jitter
* Damage → missing lines
* Signal loss → ghosting + lag

---

## 9. CAMERA & VIEW REQUIREMENTS (NEW)

### 9.1 Core Rule

ALL gameplay views must be third-person.

* No true first-person gameplay
* Avoid immersive first-person feel

---

### 9.2 Mode Cameras

Ship (External):

* Third-person view of ship
* Shows surroundings and signal sphere

Module:

* Third-person chase camera
* Slight offset for readability

Astronaut (EVA):

* Close third-person (over-shoulder or slightly pulled back)
* Astronaut body always visible

---

### 9.3 Camera Goals

* Maintain spatial awareness
* Keep player oriented at all times
* Avoid claustrophobic or disorienting perspectives

---

## 10. PROCEDURAL WORLD GENERATION (NEW REQUIREMENT)

### 10.1 Core Rule

All space environments must be procedurally generated.

---

### 10.2 Seed-Based Generation

* Each world is generated from a seed value
* Changing seed produces a different world

Seed affects:

* Star distribution
* Planet placement
* Debris fields
* Station locations
* Mission layouts

---

### 10.3 Consistency Rules

* Same seed must always generate the same world
* Different seeds must feel meaningfully different

---

### 10.4 Gameplay Preservation

Procedural generation MUST NOT break gameplay rules.

Constraints:

* Distances remain playable
* Signal ranges remain valid
* Mission objectives remain achievable

---

### 10.5 Distribution Principles

* Space is sparse but never empty
* Points of interest are always within navigable distance
* Visual anchors are always present
* Celestial anchors should support real-scale perception (large + distant, but still readable)

---

## 11. Mission Types

* Retrieval
* Navigation
* Scanning
* Repair
* Transport

---

## 12. Tension Systems

* Distance
* Fuel
* Oxygen
* Signal
* Precision

---

## 13. Failure States

* Module loss
* Astronaut loss
* Resource depletion

Penalty:

* Lose progress/resources

---

## 14. Progression

Ship:

* Fuel
* Signal
* Stability

Module:

* Control
* Durability

Astronaut:

* Tether
* Oxygen

---

## 15. Audio

* Minimal
* Engine hum
* Breathing
* Silence

---

## 16. MVP

Phase 1:

* Module movement

Phase 2:

* Ship systems

Phase 3:

* EVA + full loop

---

## 17. Key Design Challenges (Expanded)

---

### 17.1 Movement Feel (Critical System)

#### Problem
Pure physics-based movement feels:
- Floaty
- Hard to control
- Slow to correct

Arcade movement feels:
- Unrealistic
- Breaks immersion

#### Goal
A hybrid **“guided inertia” system**:
- Velocity exists
- Player intent overrides drift

#### Risks
- Overshooting targets
- Slippery/unpredictable control
- Over-reliance on boost

#### Design Levers
- Hidden drag
- Auto-stabilization strength
- Rotation responsiveness
- Max speed cap
- Input smoothing

#### Test Criteria
- Can the player stop precisely near a target?
- Can they recover from mistakes quickly?
- Does movement feel predictable?

#### Red Flag
> If players feel like they are fighting the controls, the system fails.

---

### 17.2 Visual Clarity in Wireframe Space

#### Problem
Wireframe visuals can become:
- Noisy
- Hard to read
- Depth-confusing

#### Goal
A **strict visual hierarchy system**

#### Rules
- Important objects = more detail
- Unimportant objects = simpler geometry
- Near = thicker/brighter lines
- Far = thinner/dimmer lines
- Hidden lines are NOT rendered

#### Risks
- Foreground/background confusion
- Overlapping shapes
- Visual overload

#### Test Criteria
- Can the player instantly identify:
  - Ship
  - Module
  - Objective
- Can distance be judged without UI?

#### Red Flag
> If UI is needed to clarify visuals, the rendering system is failing.

---

### 17.3 Spatial Readability & Orientation

#### Problem
Space lacks natural reference points, causing disorientation.

#### Goal
The player must **always understand orientation and movement**.

#### Requirements
At all times, the player must see:
- Starfield (parallax)
- Distant anchor objects (planets/stations)
- Optional reference grid

#### Rules
- Never allow an empty frame
- Always maintain 2–3 visible reference points

#### Risks
- Player gets lost
- Cannot tell direction or motion
- Movement feels meaningless

#### Test Criteria
- After rotating camera, can player instantly reorient?
- Can direction of movement be understood without UI?

#### Red Flag
> “Where am I?” = critical failure

---

### 17.4 Mode Switching (Cognitive Load)

#### Problem
Multiple modes (Ship / Module / EVA) increase cognitive load.

#### Goal
Switching must feel **seamless and empowering**, not disruptive.

#### Requirements
- Instant switching
- No scene resets
- Continuous world state

#### Design Principles
- Visual continuity across modes
- Clear identity per mode:
  - Ship = stable/systemic
  - Module = agile/precise
  - EVA = slow/vulnerable

#### Risks
- Player confusion
- Context loss
- Switching avoidance

#### Test Criteria
- Can player switch and instantly understand the situation?
- Does switching feel natural?

#### Red Flag
> If players hesitate to switch modes, design is failing.

---

### 17.5 Engagement Without Enemies

#### Problem
No combat removes traditional engagement loops.

#### Goal
Create **tension through systems**, not threats.

#### Sources of Tension
- Distance from safety
- Fuel limits
- Oxygen limits
- Signal degradation
- Precision navigation

#### Design Techniques
- Combine multiple constraints
- Emphasize return journey difficulty
- Introduce small complications

#### Risks
- Gameplay feels empty
- No urgency
- Repetition becomes obvious

#### Test Criteria
- Does the player feel pressure without enemies?
- Do missions create memorable situations?

#### Red Flag
> If the player feels safe all the time, engagement is lost.

---

### 17.6 Procedural Generation

#### Problem
Procedural systems can create:
- Random but meaningless layouts
- Broken or unfair scenarios

#### Goal
Generate **structured, navigable space**

#### Core Principle
> Generate relationships, not just positions.

#### Required Structure
- Anchor objects (planets/stations)
- Mid-scale structures (debris fields)
- Navigable spatial relationships

#### Constraints
- Distances must always be playable
- Orientation anchors must always exist
- Return paths must always be possible

#### Risks
- Impossible missions
- Disorientation
- Repetitive layouts

#### Test Criteria
- Is every seed playable?
- Can player always find their way back?

#### Red Flag
> If bad seeds must be discarded, generation is failing.

---

### 17.7 Signal System

#### Problem
Signal affects control, visuals, and tension simultaneously.

#### Goal
Make signal **intuitive, visible, and fair**

#### Rules
- Signal range must be visible (signal sphere)
- Degradation must be gradual
- Visual degradation happens before control loss

#### Effects
- Weak signal:
  - Flicker
  - Delay
  - Instability

#### Risks
- Feels like a bug instead of a mechanic
- Player confusion
- Unfair failure

#### Test Criteria
- Can player predict signal failure?
- Do they understand what caused failure?

#### Red Flag
> “The controls are broken” = system failure

---

## 18. Target Experience

* Calm but tense
* Minimal but deep
* Player-driven

---

## 19. Experience Goals

* "I made it back just in time"
* "I almost lost the signal"
* "I was too far out"

---

## 20. Core Identity

A fully procedural wireframe space mission game where **all visuals are generated from simple rules**, and gameplay clarity emerges from consistent rendering logic rather than art assets.
