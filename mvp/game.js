const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");

const hud = {
  phase: document.getElementById("phase"),
  mode: document.getElementById("mode"),
  resources: document.getElementById("resources"),
  status: document.getElementById("status"),
};

const clamp = (v, min, max) => Math.max(min, Math.min(max, v));
const lerp = (a, b, t) => a + (b - a) * t;
const TAU = Math.PI * 2;

function lerpAngle(a, b, t) {
  let delta = (b - a) % TAU;
  if (delta > Math.PI) delta -= TAU;
  if (delta < -Math.PI) delta += TAU;
  return a + delta * t;
}

function vec3(x = 0, y = 0, z = 0) {
  return { x, y, z };
}

function add(a, b) {
  return vec3(a.x + b.x, a.y + b.y, a.z + b.z);
}

function sub(a, b) {
  return vec3(a.x - b.x, a.y - b.y, a.z - b.z);
}

function mul(a, s) {
  return vec3(a.x * s, a.y * s, a.z * s);
}

function len(v) {
  return Math.hypot(v.x, v.y, v.z);
}

function norm(v) {
  const l = len(v) || 1;
  return vec3(v.x / l, v.y / l, v.z / l);
}

function rotateY(v, yaw) {
  const c = Math.cos(yaw);
  const s = Math.sin(yaw);
  return vec3(v.x * c - v.z * s, v.y, v.x * s + v.z * c);
}

function rgba(hex, alpha) {
  const m = hex.match(/^#(..)(..)(..)$/);
  if (!m) return `rgba(255,255,255,${alpha})`;
  const r = parseInt(m[1], 16);
  const g = parseInt(m[2], 16);
  const b = parseInt(m[3], 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function meter(value, width = 16) {
  const v = clamp(value, 0, 100);
  const filled = Math.round((v / 100) * width);
  return `${"#".repeat(filled)}${"-".repeat(width - filled)}`;
}

function hashSeed(seed) {
  let h = 2166136261;
  for (let i = 0; i < seed.length; i++) {
    h ^= seed.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

function mulberry32(seed) {
  let t = seed >>> 0;
  return function random() {
    t += 0x6d2b79f5;
    let r = Math.imul(t ^ (t >>> 15), 1 | t);
    r ^= r + Math.imul(r ^ (r >>> 7), 61 | r);
    return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
  };
}

function randomSeedString() {
  return Math.floor(Math.random() * 0xffffffff)
    .toString(36)
    .padStart(6, "0")
    .slice(0, 8);
}

const colors = {
  white: "#f4f4f4",
  red: "#ff5454",
  blue: "#58a9ff",
  green: "#4bff94",
};

const PHASE = {
  OUTBOUND: "OUTBOUND_TO_OBJECTIVE",
  RETURN: "RETURN_TO_SHIP",
  SUCCESS: "MISSION_SUCCESS",
  FAIL: "MISSION_FAIL",
};

const state = {
  seed: "",
  mode: "MODULE",
  phase: PHASE.OUTBOUND,
  message: "Fly to objective",
  time: 0,

  ship: {
    pos: vec3(0, 0, 0),
    vel: vec3(),
    yaw: 0,
    fuel: 100,
    heat: 0,
  },

  module: {
    pos: vec3(0, 0, 120),
    vel: vec3(),
    yaw: Math.PI,
    fuel: 100,
    heat: 0,
    damage: 0,
  },

  eva: {
    pos: vec3(0, 0, 130),
    vel: vec3(),
    yaw: Math.PI,
    oxygen: 100,
    deployed: false,
    tetherMax: 240,
  },

  objective: {
    pos: vec3(900, 40, -960),
    radius: 74,
  },

  signal: {
    radius: 720,
    quality: 1,
    edge: 0,
    outside: 0,
    lagFrames: 0,
  },

  camera: {
    pos: vec3(0, 120, 280),
    yaw: Math.PI,
    pitch: -0.23,
  },

  world: {
    starsSky: [],
    starsDeep: [],
    anchors: [],
    debris: [],
  },

  inputQueue: [],
};

const keys = new Set();
window.addEventListener("keydown", (event) => {
  if (event.repeat) return;
  if (event.code === "Digit1") state.mode = "SHIP";
  if (event.code === "Digit2") state.mode = "MODULE";
  if (event.code === "Digit3") {
    state.mode = "EVA";
    state.eva.deployed = true;
  }
  if (event.code === "KeyR") resetRun();
  if (event.code === "KeyT") initWorld(randomSeedString());
  keys.add(event.code);
});
window.addEventListener("keyup", (event) => keys.delete(event.code));

function getInputVec() {
  return vec3(
    (keys.has("KeyA") ? 1 : 0) - (keys.has("KeyD") ? 1 : 0),
    (keys.has("Space") ? 1 : 0) - (keys.has("ControlLeft") ? 1 : 0),
    (keys.has("KeyS") ? 1 : 0) - (keys.has("KeyW") ? 1 : 0)
  );
}

function getLaggedInput(input, lagFrames) {
  state.inputQueue.push(input);
  if (state.inputQueue.length > 64) state.inputQueue.shift();
  if (lagFrames <= 0 || state.inputQueue.length <= lagFrames) return input;
  return state.inputQueue[state.inputQueue.length - 1 - lagFrames];
}

function initWorld(seedString) {
  state.seed = seedString;
  state.mode = "MODULE";
  const rng = mulberry32(hashSeed(seedString));

  state.phase = PHASE.OUTBOUND;
  state.message = "Fly to objective";
  state.time = 0;

  state.ship.pos = vec3(0, 0, 0);
  state.ship.vel = vec3();
  state.ship.yaw = 0;
  state.ship.fuel = 100;
  state.ship.heat = 0;

  state.module.pos = vec3((rng() - 0.5) * 60, (rng() - 0.5) * 30, 120 + rng() * 40);
  state.module.vel = vec3();
  state.module.yaw = Math.PI + (rng() - 0.5) * 0.25;
  state.module.fuel = 100;
  state.module.heat = 0;
  state.module.damage = 0;

  state.eva.pos = add(state.module.pos, vec3(0, 0, 12));
  state.eva.vel = vec3();
  state.eva.yaw = state.module.yaw;
  state.eva.oxygen = 100;
  state.eva.deployed = false;

  state.signal.radius = 680 + rng() * 120;
  state.signal.quality = 1;
  state.signal.edge = 0;
  state.signal.outside = 0;
  state.signal.lagFrames = 0;

  const objectiveDist = 860 + rng() * 420;
  const objectiveAngle = rng() * TAU;
  state.objective.pos = vec3(
    Math.cos(objectiveAngle) * objectiveDist,
    (rng() - 0.5) * 120,
    Math.sin(objectiveAngle) * objectiveDist
  );
  state.objective.radius = 66 + rng() * 18;

  state.world.starsSky = Array.from({ length: 1000 }, () => {
    const u = rng();
    const v = rng();
    const theta = Math.acos(1 - 2 * u);
    const phi = v * TAU;
    return {
      dir: vec3(
        Math.sin(theta) * Math.cos(phi),
        Math.cos(theta),
        Math.sin(theta) * Math.sin(phi)
      ),
      size: rng() < 0.09 ? 2.2 : 1.2,
      brightness: 0.45 + rng() * 0.55,
      twinkle: 0.7 + rng() * 1.8,
      phase: rng() * TAU,
    };
  });

  state.world.starsDeep = [
    ...Array.from({ length: 180 }, () => ({
      layer: 0.25,
      pos: vec3((rng() - 0.5) * 9000, (rng() - 0.5) * 6500, (rng() - 0.5) * 9000),
    })),
    ...Array.from({ length: 120 }, () => ({
      layer: 0.58,
      pos: vec3((rng() - 0.5) * 7600, (rng() - 0.5) * 5300, (rng() - 0.5) * 7600),
    })),
  ];

  state.world.anchors = Array.from({ length: 4 }, (_, i) => {
    const dist = 1500 + rng() * 2200;
    const angle = (i / 4) * TAU + (rng() - 0.5) * 0.7;
    return {
      type: i % 2 === 0 ? "PLANET" : "STATION",
      pos: vec3(
        Math.cos(angle) * dist,
        (rng() - 0.5) * 420,
        Math.sin(angle) * dist
      ),
      scale: i % 2 === 0 ? 180 + rng() * 120 : 90 + rng() * 60,
    };
  });

  state.world.debris = Array.from({ length: 34 }, (_, i) => {
    const angle = (i / 34) * TAU + rng() * 0.5;
    const dist = 130 + rng() * 280;
    return vec3(
      state.objective.pos.x + Math.cos(angle) * dist,
      state.objective.pos.y + (rng() - 0.5) * 160,
      state.objective.pos.z + Math.sin(angle) * dist
    );
  });

  state.inputQueue.length = 0;

  const params = new URLSearchParams(window.location.search);
  params.set("seed", state.seed);
  window.history.replaceState({}, "", `${window.location.pathname}?${params.toString()}`);
}

function resetRun() {
  initWorld(state.seed);
}

function updateSignal() {
  const d = len(sub(state.module.pos, state.ship.pos));
  const ratio = d / state.signal.radius;
  state.signal.quality = clamp(1 - (ratio - 0.16), 0, 1);
  state.signal.edge = clamp((ratio - 0.82) / 0.2, 0, 1);
  state.signal.outside = clamp((ratio - 1) / 0.5, 0, 1);
  state.signal.lagFrames = Math.round(state.signal.outside * 14);
}

function updateMission() {
  const distObjective = len(sub(state.module.pos, state.objective.pos));
  const distShip = len(sub(state.module.pos, state.ship.pos));

  if (state.phase === PHASE.OUTBOUND && distObjective < state.objective.radius + 40) {
    state.phase = PHASE.RETURN;
    state.message = "Objective reached. Return to ship.";
  }

  if (state.phase === PHASE.RETURN && distShip < 78 && len(state.module.vel) < 36) {
    state.phase = PHASE.SUCCESS;
    state.message = "Recovered safely. Press T for a new procedural seed.";
  }

  if (state.module.damage >= 100) {
    state.phase = PHASE.FAIL;
    state.message = "Failure: module destroyed.";
  }

  if (state.phase !== PHASE.FAIL && state.module.fuel <= 0.01 && distShip > state.signal.radius * 1.35) {
    state.phase = PHASE.FAIL;
    state.message = "Failure: stranded beyond reliable signal.";
  }
}

function applyGuidedControl(entity, dt, config) {
  const rawInput = getInputVec();
  const input = getLaggedInput(rawInput, config.lagFrames || 0);
  const turn = ((keys.has("KeyE") ? 1 : 0) - (keys.has("KeyQ") ? 1 : 0)) * dt * config.turnRate;
  entity.yaw += turn;

  const boosting = keys.has("ShiftLeft") || keys.has("ShiftRight");
  const throttle = boosting ? config.boostMult : 1;

  const inputDir = len(input) > 0 ? norm(input) : vec3();
  const resourceNow = config.resourceObj[config.resourceKey];
  const resourceFactor =
    resourceNow < config.lowResourceThreshold
      ? config.lowResourceMin + (resourceNow / config.lowResourceThreshold) * (1 - config.lowResourceMin)
      : 1;

  const accel = config.baseAccel * throttle * resourceFactor;
  const worldInput = rotateY(inputDir, entity.yaw);
  entity.vel = add(entity.vel, mul(worldInput, accel * dt));
  entity.vel = mul(entity.vel, Math.exp(-config.drag * dt));

  if (len(input) === 0) {
    entity.vel = mul(entity.vel, Math.exp(-config.assist * dt));
  }

  const maxSpeed = boosting ? config.maxSpeedBoost : config.maxSpeed;
  const speed = len(entity.vel);
  if (speed > maxSpeed) entity.vel = mul(norm(entity.vel), maxSpeed);

  const usage = len(input) * dt * (boosting ? config.drainBoost : config.drain);
  config.resourceObj[config.resourceKey] = clamp(resourceNow - usage, 0, 100);
  if (config.resourceObj[config.resourceKey] <= 0.01) {
    entity.vel = mul(entity.vel, Math.exp(-0.65 * dt));
  }

  entity.heat = clamp((entity.heat || 0) + usage * 0.012 - dt * 0.08, 0, 1);
}

function updateWorldPhysics(dt) {
  if (state.signal.outside > 0.01) {
    const jitter = vec3(
      Math.sin(state.time * 18.3),
      Math.cos(state.time * 10.1),
      Math.sin(state.time * 13.4)
    );
    state.module.vel = add(state.module.vel, mul(jitter, state.signal.outside * 14 * dt));
  }

  state.module.pos = add(state.module.pos, mul(state.module.vel, dt));
  state.ship.pos = add(state.ship.pos, mul(state.ship.vel, dt));
  state.eva.pos = add(state.eva.pos, mul(state.eva.vel, dt));

  if (!state.eva.deployed) {
    state.eva.pos = add(state.module.pos, vec3(0, 0, 12));
    state.eva.vel = mul(state.module.vel, 0.5);
    state.eva.yaw = state.module.yaw;
  } else {
    const tether = sub(state.eva.pos, state.module.pos);
    const tetherLen = len(tether);
    if (tetherLen > state.eva.tetherMax) {
      const d = norm(tether);
      state.eva.pos = add(state.module.pos, mul(d, state.eva.tetherMax));
      state.eva.vel = mul(state.eva.vel, 0.82);
      state.eva.oxygen = clamp(state.eva.oxygen - dt * 1.6, 0, 100);
    }
  }

  for (const rock of state.world.debris) {
    const d = len(sub(state.module.pos, rock));
    const speed = len(state.module.vel);
    if (d < 24 && speed > 70) {
      state.module.damage = clamp(state.module.damage + dt * speed * 0.58, 0, 100);
      state.module.heat = clamp(state.module.heat + dt * 0.5, 0, 1);
    }
  }
}

function update(dt) {
  state.time += dt;

  if (state.phase !== PHASE.SUCCESS && state.phase !== PHASE.FAIL) {
    updateSignal();
    if (state.mode === "MODULE") {
      applyGuidedControl(state.module, dt, {
        turnRate: 2.2,
        boostMult: 1.85,
        baseAccel: 180,
        drag: 0.92,
        assist: 2.35,
        maxSpeed: 220,
        maxSpeedBoost: 300,
        drain: 4.6,
        drainBoost: 8.2,
        lowResourceThreshold: 25,
        lowResourceMin: 0.35,
        lagFrames: state.signal.lagFrames,
        resourceObj: state.module,
        resourceKey: "fuel",
      });
    } else if (state.mode === "SHIP") {
      applyGuidedControl(state.ship, dt, {
        turnRate: 1.6,
        boostMult: 1.4,
        baseAccel: 120,
        drag: 0.8,
        assist: 1.8,
        maxSpeed: 170,
        maxSpeedBoost: 220,
        drain: 2.2,
        drainBoost: 3.6,
        lowResourceThreshold: 20,
        lowResourceMin: 0.5,
        lagFrames: 0,
        resourceObj: state.ship,
        resourceKey: "fuel",
      });
    } else {
      state.eva.deployed = true;
      applyGuidedControl(state.eva, dt, {
        turnRate: 1.8,
        boostMult: 1.3,
        baseAccel: 90,
        drag: 1.1,
        assist: 2.8,
        maxSpeed: 130,
        maxSpeedBoost: 170,
        drain: 2.5,
        drainBoost: 4,
        lowResourceThreshold: 30,
        lowResourceMin: 0.45,
        lagFrames: 0,
        resourceObj: state.eva,
        resourceKey: "oxygen",
      });
      state.eva.oxygen = clamp(state.eva.oxygen - dt * 0.35, 0, 100);
    }

    updateWorldPhysics(dt);
    updateMission();

    if (state.signal.outside > 0.72) {
      state.message = "Signal degrading: delay and drift increasing.";
    } else if (state.signal.edge > 0.18 && state.phase !== PHASE.RETURN) {
      state.message = "Signal edge: unstable.";
    }
  } else {
    state.module.vel = mul(state.module.vel, Math.exp(-1.2 * dt));
    state.module.pos = add(state.module.pos, mul(state.module.vel, dt));
    state.ship.vel = mul(state.ship.vel, Math.exp(-1.2 * dt));
    state.ship.pos = add(state.ship.pos, mul(state.ship.vel, dt));
    state.eva.vel = mul(state.eva.vel, Math.exp(-1.4 * dt));
    state.eva.pos = add(state.eva.pos, mul(state.eva.vel, dt));
  }
}

function project(point, camera) {
  const rel = sub(point, camera.pos);
  const yaw = -camera.yaw;
  const pitch = -camera.pitch;

  const cy = Math.cos(yaw);
  const sy = Math.sin(yaw);
  let x = rel.x * cy - rel.z * sy;
  let z = rel.x * sy + rel.z * cy;

  const cp = Math.cos(pitch);
  const sp = Math.sin(pitch);
  const y = rel.y * cp - z * sp;
  z = rel.y * sp + z * cp;

  if (z < 1.2) return null;

  const f = camera.fov / z;
  return {
    x: canvas.width * 0.5 + x * f,
    y: canvas.height * 0.5 - y * f,
    z,
  };
}

function drawLine3D(a, b, camera, color, alpha = 1, baseWidth = 1) {
  const pa = project(a, camera);
  const pb = project(b, camera);
  if (!pa || !pb) return;

  const depth = (pa.z + pb.z) * 0.5;
  const fade = clamp(1 - depth / 3000, 0.06, 1);
  const flicker = state.signal.edge > 0.15 ? 1 - state.signal.edge * (Math.random() * 0.24) : 1;

  ctx.strokeStyle = rgba(color, alpha * fade * flicker);
  ctx.lineWidth = baseWidth + clamp(4 / (depth * 0.011 + 1), 0.45, 2.7);
  ctx.beginPath();
  ctx.moveTo(pa.x, pa.y);
  ctx.lineTo(pb.x, pb.y);
  ctx.stroke();
}

function drawPoint3D(point, camera, color, alpha = 1, size = 2) {
  const p = project(point, camera);
  if (!p) return;
  const fade = clamp(1 - p.z / 3800, 0.16, 1);
  const s = clamp(size / (p.z * 0.005 + 1), 0.9, size + 1.8);
  ctx.fillStyle = rgba(color, alpha * fade);
  ctx.fillRect(p.x, p.y, s, s);
}

function drawObject(pos, yaw, scale, verts, edges, color, camera, options = {}) {
  const jitter = options.jitter || 0;
  const skipChance = options.skipChance || 0;

  const pts = verts.map((v) => {
    const rv = rotateY(mul(v, scale), yaw);
    return add(
      pos,
      vec3(
        rv.x + (Math.random() - 0.5) * jitter,
        rv.y + (Math.random() - 0.5) * jitter,
        rv.z + (Math.random() - 0.5) * jitter
      )
    );
  });

  for (const [a, b] of edges) {
    if (Math.random() < skipChance) continue;
    drawLine3D(pts[a], pts[b], camera, color, options.alpha ?? 0.95, options.baseWidth ?? 1);
  }
}

function drawStarfield(camera) {
  const yaw = -camera.yaw * 0.92;
  const pitch = -camera.pitch * 0.92;
  const cy = Math.cos(yaw);
  const sy = Math.sin(yaw);
  const cp = Math.cos(pitch);
  const sp = Math.sin(pitch);

  for (const star of state.world.starsSky) {
    const d = star.dir;
    let x = d.x * cy - d.z * sy;
    let z = d.x * sy + d.z * cy;
    const y = d.y * cp - z * sp;
    z = d.y * sp + z * cp;

    if (z <= 0.02) continue;

    const f = (camera.fov * 1.3) / z;
    const sx = canvas.width * 0.5 + x * f;
    const sy2 = canvas.height * 0.5 - y * f;
    if (sx < -40 || sy2 < -40 || sx > canvas.width + 40 || sy2 > canvas.height + 40) continue;

    const twinkle = 0.74 + 0.26 * Math.sin(state.time * star.twinkle + star.phase);
    const a = clamp(star.brightness * twinkle, 0.25, 1);
    const s = star.size * (0.85 + z * 0.8);
    ctx.fillStyle = rgba(colors.white, a);
    ctx.fillRect(sx, sy2, s, s);
  }

  for (const star of state.world.starsDeep) {
    const drifted = sub(star.pos, mul(camera.pos, star.layer * 0.35));
    drawPoint3D(drifted, camera, colors.white, 0.72, 2.1);
  }
}

function drawGrid(camera) {
  const span = 16;
  const step = 130;
  for (let i = -span; i <= span; i++) {
    drawLine3D(vec3(-span * step, -110, i * step), vec3(span * step, -110, i * step), camera, colors.white, 0.1, 0.32);
    drawLine3D(vec3(i * step, -110, -span * step), vec3(i * step, -110, span * step), camera, colors.white, 0.1, 0.32);
  }
}

function drawSignalSphere(camera) {
  const color = state.signal.outside > 0.02 ? colors.red : colors.green;
  const r = state.signal.radius;
  const seg = 44;

  for (let i = 0; i < seg; i++) {
    const a0 = (i / seg) * TAU;
    const a1 = ((i + 1) / seg) * TAU;

    drawLine3D(add(state.ship.pos, vec3(Math.cos(a0) * r, 0, Math.sin(a0) * r)), add(state.ship.pos, vec3(Math.cos(a1) * r, 0, Math.sin(a1) * r)), camera, color, 0.16, 0.45);
    drawLine3D(add(state.ship.pos, vec3(0, Math.cos(a0) * r, Math.sin(a0) * r)), add(state.ship.pos, vec3(0, Math.cos(a1) * r, Math.sin(a1) * r)), camera, color, 0.1, 0.4);
  }
}

function drawAnchors(camera) {
  for (const anchor of state.world.anchors) {
    if (anchor.type === "PLANET") {
      const seg = 34;
      for (let i = 0; i < seg; i++) {
        const a0 = (i / seg) * TAU;
        const a1 = ((i + 1) / seg) * TAU;
        drawLine3D(
          add(anchor.pos, vec3(Math.cos(a0) * anchor.scale, 0, Math.sin(a0) * anchor.scale)),
          add(anchor.pos, vec3(Math.cos(a1) * anchor.scale, 0, Math.sin(a1) * anchor.scale)),
          camera,
          colors.white,
          0.28,
          0.5
        );
      }
    } else {
      const verts = [
        vec3(-1, -1, -1), vec3(1, -1, -1), vec3(1, 1, -1), vec3(-1, 1, -1),
        vec3(-1, -1, 1), vec3(1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1),
      ];
      const edges = [[0,1],[1,2],[2,3],[3,0],[4,5],[5,6],[6,7],[7,4],[0,4],[1,5],[2,6],[3,7]];
      drawObject(anchor.pos, state.time * 0.08, anchor.scale, verts, edges, colors.white, camera, { alpha: 0.22, baseWidth: 0.45 });
    }
  }
}

function drawDebris(camera) {
  for (const d of state.world.debris) {
    const tri = [add(d, vec3(-13, -9, 0)), add(d, vec3(13, -9, 0)), add(d, vec3(0, 12, 0))];
    drawLine3D(tri[0], tri[1], camera, colors.white, 0.52, 0.72);
    drawLine3D(tri[1], tri[2], camera, colors.white, 0.52, 0.72);
    drawLine3D(tri[2], tri[0], camera, colors.white, 0.52, 0.72);
  }
}

function drawShip(camera) {
  const verts = [
    vec3(0, 0, -1.8), vec3(1, 0.2, 0.72), vec3(-1, 0.2, 0.72), vec3(0, -0.35, 1.25),
    vec3(0, 0.85, 0.48), vec3(0, -0.9, 0.35), vec3(1.55, 0, 0.95), vec3(-1.55, 0, 0.95),
  ];
  const edges = [[0,1],[0,2],[1,3],[2,3],[1,4],[2,4],[1,6],[2,7],[6,3],[7,3],[5,3],[5,0]];
  drawObject(state.ship.pos, 0, 36, verts, edges, colors.white, camera, { alpha: 0.95, baseWidth: 1 });
}

function drawModule(camera) {
  const verts = [
    vec3(0, 0, -1.35), vec3(1.02, 0, 0), vec3(0, 0, 1.2), vec3(-1.02, 0, 0), vec3(0, 0.64, 0), vec3(0, -0.64, 0),
  ];
  const edges = [[0,1],[1,2],[2,3],[3,0],[0,4],[1,4],[2,4],[3,4],[0,5],[1,5],[2,5],[3,5]];

  const skip = (state.module.damage / 100) * 0.38;
  const jitter = state.module.heat * 4 + (state.module.damage / 100) * 3;

  drawObject(state.module.pos, state.module.yaw, 26, verts, edges, colors.white, camera, {
    alpha: 0.95,
    baseWidth: 1.25,
    skipChance: skip,
    jitter,
  });

  const speed = len(state.module.vel);
  if (speed > 8) {
    const rear = add(state.module.pos, rotateY(vec3(0, 0, 22), state.module.yaw));
    const burnLen = (speed * 0.14) * (state.module.fuel < 20 ? 0.45 : 1);
    drawLine3D(add(rear, vec3(-6, 0, 0)), add(rear, rotateY(vec3(-6, 0, burnLen), state.module.yaw)), camera, colors.white, 0.76, 0.7);
    drawLine3D(add(rear, vec3(6, 0, 0)), add(rear, rotateY(vec3(6, 0, burnLen), state.module.yaw)), camera, colors.white, 0.76, 0.7);
  }

  if (state.signal.outside > 0.02) {
    const ghost = add(state.module.pos, vec3(
      Math.sin(state.time * 9.1) * (24 + state.signal.outside * 12),
      Math.cos(state.time * 8.2) * (14 + state.signal.outside * 9),
      Math.sin(state.time * 11.4) * (20 + state.signal.outside * 10)
    ));
    drawObject(ghost, state.module.yaw, 18, [vec3(0,0,-1.2),vec3(1,0,0),vec3(0,0,1.1),vec3(-1,0,0)], [[0,1],[1,2],[2,3],[3,0]], colors.white, camera, {
      alpha: 0.2,
      baseWidth: 0.55,
    });
  }

  // Persistent visibility aid: a small ring around the module body so it never disappears in star noise.
  const ringR = 34;
  const seg = 20;
  for (let i = 0; i < seg; i++) {
    const a0 = (i / seg) * TAU;
    const a1 = ((i + 1) / seg) * TAU;
    drawLine3D(
      add(state.module.pos, vec3(Math.cos(a0) * ringR, 0, Math.sin(a0) * ringR)),
      add(state.module.pos, vec3(Math.cos(a1) * ringR, 0, Math.sin(a1) * ringR)),
      camera,
      colors.green,
      0.34,
      0.65
    );
  }
}

function drawEVA(camera) {
  if (!state.eva.deployed && state.mode !== "EVA") return;
  const verts = [vec3(0, 0, 0), vec3(0, 1, 0), vec3(-0.6, 0, 0), vec3(0.6, 0, 0), vec3(0, -0.85, 0)];
  const edges = [[0,1],[0,2],[0,3],[0,4],[2,4],[3,4]];
  const color = state.eva.oxygen < 28 ? colors.red : colors.green;
  drawObject(state.eva.pos, state.eva.yaw, 12, verts, edges, color, camera, { alpha: 0.96, baseWidth: 1 });
}

function drawTether(camera) {
  if (!state.eva.deployed) return;
  const t = sub(state.eva.pos, state.module.pos);
  const l = len(t);
  const stress = clamp((l - state.eva.tetherMax * 0.75) / (state.eva.tetherMax * 0.25), 0, 1);
  const color = stress > 0.5 ? colors.red : colors.white;
  drawLine3D(state.eva.pos, state.module.pos, camera, color, 0.9, 0.75 + stress * 0.8);
}

function drawObjective(camera) {
  const cube = [
    vec3(-1, -1, -1), vec3(1, -1, -1), vec3(1, 1, -1), vec3(-1, 1, -1),
    vec3(-1, -1, 1), vec3(1, -1, 1), vec3(1, 1, 1), vec3(-1, 1, 1),
  ];
  const edges = [[0,1],[1,2],[2,3],[3,0],[4,5],[5,6],[6,7],[7,4],[0,4],[1,5],[2,6],[3,7]];
  drawObject(state.objective.pos, state.time * 0.22, state.objective.radius, cube, edges, colors.blue, camera, { alpha: 0.95, baseWidth: 1 });

  drawLine3D(add(state.objective.pos, vec3(0, -150, 0)), add(state.objective.pos, vec3(0, 150, 0)), camera, colors.blue, 0.34, 0.72);
}

function activeEntity() {
  if (state.mode === "SHIP") return state.ship;
  if (state.mode === "EVA") return state.eva;
  return state.module;
}

function cameraRigForMode(mode) {
  if (mode === "SHIP") {
    return { back: 380, side: 48, height: 180, fovScale: 0.76, posLerp: 0.18 };
  }
  if (mode === "EVA") {
    return { back: 200, side: 24, height: 105, fovScale: 0.9, posLerp: 0.2 };
  }
  return { back: 300, side: 36, height: 130, fovScale: 0.82, posLerp: 0.2 };
}

function drawTrajectory(camera) {
  let p = { ...state.module.pos };
  let v = { ...state.module.vel };

  for (let i = 0; i < 32; i++) {
    const n = add(p, mul(v, 0.12));
    if (i % 2 === 0) {
      drawLine3D(p, n, camera, colors.blue, 0.85 - i / 36, 0.62);
    }
    p = n;
    v = mul(v, 0.93);
  }
}

function drawReticle() {
  const cx = canvas.width * 0.5;
  const cy = canvas.height * 0.5;
  const size = 8;
  ctx.strokeStyle = rgba(colors.white, 0.32);
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(cx - size, cy);
  ctx.lineTo(cx + size, cy);
  ctx.moveTo(cx, cy - size);
  ctx.lineTo(cx, cy + size);
  ctx.stroke();
}

function drawObjectiveLabel(camera) {
  const p = project(state.objective.pos, camera);
  if (!p) return;
  const d = len(sub(state.module.pos, state.objective.pos));
  ctx.fillStyle = rgba(colors.blue, 0.95);
  ctx.font = `${12 * devicePixelRatio}px "IBM Plex Mono", monospace`;
  ctx.fillText(`OBJ ${d.toFixed(0)}m`, p.x + 8, p.y - 8);
}

function render() {
  const actor = activeEntity();
  const rig = cameraRigForMode(state.mode);
  const forward = rotateY(vec3(0, 0, -1), actor.yaw);
  const right = rotateY(vec3(1, 0, 0), actor.yaw);
  const desiredPos = add(
    actor.pos,
    add(add(mul(forward, -rig.back), mul(right, rig.side)), vec3(0, rig.height, 0))
  );
  state.camera.pos = vec3(
    lerp(state.camera.pos.x, desiredPos.x, rig.posLerp),
    lerp(state.camera.pos.y, desiredPos.y, rig.posLerp),
    lerp(state.camera.pos.z, desiredPos.z, rig.posLerp)
  );

  // Exact look-at using the same sign convention as project(), guaranteeing module stays in view.
  const lookTarget = add(actor.pos, vec3(0, 8, 0));
  const lookDir = sub(lookTarget, state.camera.pos);
  const flat = Math.max(0.0001, Math.hypot(lookDir.x, lookDir.z));
  const targetYaw = Math.atan2(-lookDir.x, lookDir.z);
  const targetPitch = Math.atan2(-lookDir.y, flat);
  state.camera.yaw = lerpAngle(state.camera.yaw, targetYaw, 0.28);
  state.camera.pitch = lerp(state.camera.pitch, targetPitch, 0.26);

  const camera = {
    pos: state.camera.pos,
    yaw: state.camera.yaw,
    pitch: state.camera.pitch,
    fov: Math.min(canvas.width, canvas.height) * rig.fovScale,
  };

  ctx.fillStyle = "#000";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  drawStarfield(camera);
  drawGrid(camera);
  drawAnchors(camera);
  drawSignalSphere(camera);
  drawDebris(camera);
  drawShip(camera);
  drawObjective(camera);
  drawModule(camera);
  drawEVA(camera);
  drawTether(camera);
  if (state.mode === "MODULE") drawTrajectory(camera);

  drawObjectiveLabel(camera);
  drawReticle();
}

function phaseLabel(phase) {
  if (phase === PHASE.OUTBOUND) return "Outbound to objective";
  if (phase === PHASE.RETURN) return "Return to ship";
  if (phase === PHASE.SUCCESS) return "Mission success";
  return "Mission fail";
}

function updateHud() {
  hud.phase.textContent = `PHASE   ${phaseLabel(state.phase)}  |  Seed ${state.seed}`;
  hud.mode.textContent = `MODE    ${state.mode} (3rd person)  |  Signal lag ${state.signal.lagFrames}f`;
  hud.resources.textContent = `SHIP FUEL [${meter(state.ship.fuel)}] ${state.ship.fuel.toFixed(0)}\nMODULE   [${meter(state.module.fuel)}] ${state.module.fuel.toFixed(0)}\nEVA O2   [${meter(state.eva.oxygen)}] ${state.eva.oxygen.toFixed(0)}\nSIGNAL   [${meter(state.signal.quality * 100)}] ${(state.signal.quality * 100).toFixed(0)}%\nDAMAGE   [${meter(100 - state.module.damage)}] ${state.module.damage.toFixed(0)}%`;
  hud.status.textContent = `STATUS  ${state.message}`;
}

function resize() {
  canvas.width = Math.floor(window.innerWidth * devicePixelRatio);
  canvas.height = Math.floor(window.innerHeight * devicePixelRatio);
  canvas.style.width = `${window.innerWidth}px`;
  canvas.style.height = `${window.innerHeight}px`;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
}
window.addEventListener("resize", resize);
resize();

const querySeed = new URLSearchParams(window.location.search).get("seed");
initWorld(querySeed || randomSeedString());

let last = performance.now();
function frame(now) {
  const dt = Math.min((now - last) / 1000, 0.033);
  last = now;

  update(dt);
  render();
  updateHud();

  requestAnimationFrame(frame);
}
requestAnimationFrame(frame);
