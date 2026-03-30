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

function clampCruiseSpeed(v) {
  return clamp(v, 30, 4000);
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

const SHIP_NAV = {
  CRUISE: "CRUISE",
  ORBIT: "ORBIT",
  STANDBY: "STANDBY",
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
    cruiseYaw: 0,
    cruiseSpeed: 111120,
    distanceTraveled: 0,
    navState: SHIP_NAV.CRUISE,
    systems: {
      reactor: 62,
      comms: 58,
      thermal: 54,
      propulsion: 66,
    },
  },

  module: {
    pos: vec3(0, 0, 120),
    vel: vec3(),
    yaw: Math.PI,
    fuel: 100,
    heat: 0,
    damage: 0,
    attached: true,
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
    solarSystem: null,
    anchors: [],
    debris: [],
  },

  inputQueue: [],
  mouseLook: {
    dragging: false,
    lastX: 0,
    lastY: 0,
    yawOffset: 0,
    pitchOffset: 0,
  },
};

const keys = new Set();
window.addEventListener("keydown", (event) => {
  if (event.repeat) return;
  if (event.code === "Digit1") state.mode = "SHIP_INT";
  if (event.code === "Digit2") state.mode = "MODULE";
  if (event.code === "Digit3") state.mode = "EVA";
  if (event.code === "Digit4") state.mode = "SHIP";
  if (event.code === "BracketRight" || event.code === "Equal" || event.code === "NumpadAdd") {
    state.ship.cruiseSpeed = clampCruiseSpeed(state.ship.cruiseSpeed + 120);
    state.message = `Ship cruise setpoint: ${state.ship.cruiseSpeed.toFixed(0)} u/s`;
  }
  if (event.code === "BracketLeft" || event.code === "Minus" || event.code === "NumpadSubtract") {
    state.ship.cruiseSpeed = clampCruiseSpeed(state.ship.cruiseSpeed - 120);
    state.message = `Ship cruise setpoint: ${state.ship.cruiseSpeed.toFixed(0)} u/s`;
  }
  if (event.code === "KeyM") {
    const order = [SHIP_NAV.CRUISE, SHIP_NAV.ORBIT, SHIP_NAV.STANDBY];
    const next = (order.indexOf(state.ship.navState) + 1) % order.length;
    state.ship.navState = order[next];
  }
  if (event.code === "KeyX") {
    if (state.module.attached) {
      if (state.ship.navState !== SHIP_NAV.CRUISE) {
        state.module.attached = false;
        state.message = "Explorer detached.";
      } else {
        state.message = "Cannot detach while ship is cruising. Set ORBIT/STANDBY first.";
      }
    } else {
      const near = len(sub(state.module.pos, state.ship.pos)) < 95;
      const rel = len(sub(state.module.vel, state.ship.vel)) < 24;
      if (near && rel) {
        state.module.attached = true;
        state.module.vel = { ...state.ship.vel };
        state.module.yaw = state.ship.yaw;
        state.message = "Explorer reattached.";
      } else {
        state.message = "Move explorer near ship and match velocity to reattach.";
      }
    }
  }
  if (event.code === "KeyC") {
    if (!state.eva.deployed) {
      state.eva.deployed = true;
      const evaOffset = rotateY(vec3(0, 0, 12), state.module.yaw);
      state.eva.pos = add(state.module.pos, evaOffset);
      state.eva.vel = { ...state.module.vel };
      state.eva.yaw = state.module.yaw;
      state.message = "Astronaut detached (tether active).";
    } else {
      const near = len(sub(state.eva.pos, state.module.pos)) < 35;
      const rel = len(sub(state.eva.vel, state.module.vel)) < 22;
      if (near && rel) {
        state.eva.deployed = false;
        state.eva.pos = add(state.module.pos, vec3(0, 0, 12));
        state.eva.vel = { ...state.module.vel };
        state.eva.yaw = state.module.yaw;
        state.message = "Astronaut recovered.";
      } else {
        state.message = "Move astronaut near module and match velocity to recover.";
      }
    }
  }
  if (event.code === "KeyR") resetRun();
  if (event.code === "KeyT") initWorld(randomSeedString());
  keys.add(event.code);
});
window.addEventListener("keyup", (event) => keys.delete(event.code));
window.addEventListener("mousedown", (event) => {
  if (event.button !== 0) return;
  state.mouseLook.dragging = true;
  state.mouseLook.lastX = event.clientX;
  state.mouseLook.lastY = event.clientY;
});
window.addEventListener("mouseup", (event) => {
  if (event.button !== 0) return;
  state.mouseLook.dragging = false;
});
window.addEventListener("mousemove", (event) => {
  if (!state.mouseLook.dragging) return;
  const dx = event.clientX - state.mouseLook.lastX;
  const dy = event.clientY - state.mouseLook.lastY;
  state.mouseLook.lastX = event.clientX;
  state.mouseLook.lastY = event.clientY;
  state.mouseLook.yawOffset = (state.mouseLook.yawOffset - dx * 0.0055) % TAU;
  state.mouseLook.pitchOffset = clamp(state.mouseLook.pitchOffset - dy * 0.0035, -0.75, 0.75);
});
window.addEventListener("blur", () => {
  state.mouseLook.dragging = false;
});

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
  state.ship.cruiseYaw = rng() * TAU;
  state.ship.cruiseSpeed = 100 + rng() * 40;
  state.ship.distanceTraveled = 0;
  state.ship.navState = SHIP_NAV.CRUISE;
  state.ship.systems = {
    reactor: 58 + rng() * 20,
    comms: 52 + rng() * 18,
    thermal: 50 + rng() * 16,
    propulsion: 60 + rng() * 22,
  };
  const cruiseForward = rotateY(vec3(0, 0, -1), state.ship.cruiseYaw);
  state.ship.vel = mul(cruiseForward, state.ship.cruiseSpeed);
  state.ship.yaw = state.ship.cruiseYaw;

  state.module.pos = vec3((rng() - 0.5) * 60, (rng() - 0.5) * 30, 120 + rng() * 40);
  state.module.vel = { ...state.ship.vel };
  state.module.yaw = Math.PI + (rng() - 0.5) * 0.25;
  state.module.fuel = 100;
  state.module.heat = 0;
  state.module.damage = 0;
  state.module.attached = true;

  state.eva.pos = add(state.module.pos, vec3(0, 0, 12));
  state.eva.vel = { ...state.ship.vel };
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

  const solarForward = rotateY(vec3(0, 0, -1), state.ship.cruiseYaw);
  const cruiseRight = rotateY(vec3(1, 0, 0), state.ship.cruiseYaw);
  const solarForwardDist = 12000 + rng() * 7000;
  const solarSideOffset = (rng() - 0.5) * 4800;
  const solarCenter = add(
    state.ship.pos,
    add(
      mul(solarForward, solarForwardDist),
      vec3(cruiseRight.x * solarSideOffset, -60 + (rng() - 0.5) * 180, cruiseRight.z * solarSideOffset)
    )
  );
  const planetCount = 3 + Math.floor(rng() * 2);
  state.world.solarSystem = {
    center: solarCenter,
    starRadius: 1800 + rng() * 1000,
    planets: Array.from({ length: planetCount }, (_, i) => ({
      orbitRadius: 5200 + i * (3600 + rng() * 900),
      size: 700 + rng() * 550,
      angularSpeed: 0.0016 + rng() * 0.003,
      phase: rng() * TAU,
      tilt: (rng() - 0.5) * 0.55,
    })),
  };

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

function updateShipCruise(dt) {
  const reactorFactor = clamp(state.ship.systems.reactor / 100, 0.45, 1.3);
  const propFactor = clamp(state.ship.systems.propulsion / 100, 0.45, 1.3);
  let targetVel = vec3(0, 0, 0);

  if (state.ship.navState === SHIP_NAV.CRUISE) {
    const forward = rotateY(vec3(0, 0, -1), state.ship.cruiseYaw);
    const cruiseTarget = state.ship.cruiseSpeed * reactorFactor * propFactor;
    targetVel = mul(forward, cruiseTarget);
    state.ship.yaw = state.ship.cruiseYaw;
  } else if (state.ship.navState === SHIP_NAV.ORBIT) {
    state.ship.yaw += dt * 0.26;
    const forward = rotateY(vec3(0, 0, -1), state.ship.yaw);
    const orbitSpeed = state.ship.cruiseSpeed * 0.44 * reactorFactor;
    targetVel = mul(forward, orbitSpeed);
  } else {
    targetVel = vec3(0, 0, 0);
  }

  state.ship.vel = vec3(
    lerp(state.ship.vel.x, targetVel.x, 0.35 * dt),
    lerp(state.ship.vel.y, targetVel.y, 0.2 * dt),
    lerp(state.ship.vel.z, targetVel.z, 0.35 * dt)
  );
  state.ship.distanceTraveled += len(state.ship.vel) * dt;
}

function updateShipInternalSystems(dt) {
  const rate = 20 * dt;
  if (keys.has("KeyU")) state.ship.systems.reactor = clamp(state.ship.systems.reactor + rate, 0, 100);
  if (keys.has("KeyJ")) state.ship.systems.reactor = clamp(state.ship.systems.reactor - rate, 0, 100);
  if (keys.has("KeyI")) state.ship.systems.comms = clamp(state.ship.systems.comms + rate, 0, 100);
  if (keys.has("KeyK")) state.ship.systems.comms = clamp(state.ship.systems.comms - rate, 0, 100);
  if (keys.has("KeyO")) state.ship.systems.thermal = clamp(state.ship.systems.thermal + rate, 0, 100);
  if (keys.has("KeyL")) state.ship.systems.thermal = clamp(state.ship.systems.thermal - rate, 0, 100);
  // Propulsion follows reactor/thermal balance so internal management impacts cruise quality.
  const thermalPenalty = clamp((state.ship.systems.thermal - 75) / 25, 0, 1) * 18;
  state.ship.systems.propulsion = clamp(
    lerp(state.ship.systems.propulsion, (state.ship.systems.reactor * 0.7 + state.ship.systems.comms * 0.3) - thermalPenalty, 0.08),
    0,
    100
  );
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

  state.ship.pos = add(state.ship.pos, mul(state.ship.vel, dt));
  if (!state.module.attached) {
    state.module.pos = add(state.module.pos, mul(state.module.vel, dt));
  }
  state.eva.pos = add(state.eva.pos, mul(state.eva.vel, dt));

  if (state.module.attached) {
    const dockOffset = rotateY(vec3(0, 0, 90), state.ship.yaw);
    state.module.pos = add(state.ship.pos, dockOffset);
    state.module.vel = { ...state.ship.vel };
    state.module.yaw = state.ship.yaw;
  }

  if (!state.eva.deployed) {
    const evaOffset = rotateY(vec3(0, 0, 12), state.module.yaw);
    state.eva.pos = add(state.module.pos, evaOffset);
    state.eva.vel = { ...state.module.vel };
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
  updateShipCruise(dt);
  if (state.mode === "SHIP_INT" || state.mode === "SHIP") {
    updateShipInternalSystems(dt);
  }

  if (state.phase !== PHASE.SUCCESS && state.phase !== PHASE.FAIL) {
    updateSignal();
    if (state.mode === "MODULE" && !state.module.attached) {
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
    } else if (state.mode === "MODULE" && state.module.attached) {
      state.module.vel = { ...state.ship.vel };
    } else if (state.mode === "EVA" && state.eva.deployed) {
      applyGuidedControl(state.eva, dt, {
        turnRate: 1.8,
        boostMult: 1.3,
        baseAccel: 90,
        drag: 0.05,
        assist: 0.12,
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
    } else if (state.mode === "MODULE" && state.module.attached) {
      state.message = "Explorer attached. Set ship to ORBIT/STANDBY, then press X to detach.";
    } else if (state.mode === "EVA" && !state.eva.deployed) {
      state.message = "Astronaut is inside module. Press C to detach (tethered).";
    } else if (state.mode === "SHIP_INT") {
      state.message = `Ship cruising at ${len(state.ship.vel).toFixed(0)} u/s. Balance reactor/comms/thermal.`;
    } else if (state.mode === "SHIP") {
      state.message = `External ship view. Cruise ${len(state.ship.vel).toFixed(0)} u/s.`;
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

function drawSolarSystem(camera) {
  const solar = state.world.solarSystem;
  if (!solar) return;

  const starPos = solar.center;
  const starRadius = solar.starRadius;
  // Sun rendered as a sphere silhouette (solid black fill + bright rim).
  const sun2 = project(starPos, camera);
  const sunEdge2 = project(add(starPos, vec3(starRadius, 0, 0)), camera);
  if (sun2 && sunEdge2) {
    const sunR = Math.max(3, Math.hypot(sunEdge2.x - sun2.x, sunEdge2.y - sun2.y));
    ctx.fillStyle = "#000000";
    ctx.beginPath();
    ctx.arc(sun2.x, sun2.y, sunR, 0, TAU);
    ctx.fill();

    ctx.strokeStyle = rgba(colors.white, 0.98);
    ctx.lineWidth = 1.6 + clamp(4 / (sun2.z * 0.011 + 1), 0.4, 2.4);
    ctx.beginPath();
    ctx.arc(sun2.x, sun2.y, sunR, 0, TAU);
    ctx.stroke();
  }

  for (let i = 0; i < solar.planets.length; i++) {
    const p = solar.planets[i];
    const orbitSeg = 44;
    for (let j = 0; j < orbitSeg; j++) {
      const a0 = (j / orbitSeg) * TAU;
      const a1 = ((j + 1) / orbitSeg) * TAU;
      const oy0 = Math.sin(a0) * p.orbitRadius * Math.sin(p.tilt);
      const oy1 = Math.sin(a1) * p.orbitRadius * Math.sin(p.tilt);
      drawLine3D(
        add(starPos, vec3(Math.cos(a0) * p.orbitRadius, oy0, Math.sin(a0) * p.orbitRadius)),
        add(starPos, vec3(Math.cos(a1) * p.orbitRadius, oy1, Math.sin(a1) * p.orbitRadius)),
        camera,
        colors.white,
        0.2,
        0.45
      );
    }

    const t = state.time * p.angularSpeed + p.phase;
    const planetPos = add(
      starPos,
      vec3(
        Math.cos(t) * p.orbitRadius,
        Math.sin(t) * p.orbitRadius * Math.sin(p.tilt),
        Math.sin(t) * p.orbitRadius
      )
    );

    const planetColor = i % 2 === 0 ? colors.blue : colors.green;
    const center2 = project(planetPos, camera);
    const edge2 = project(add(planetPos, vec3(p.size, 0, 0)), camera);
    if (!center2 || !edge2) continue;

    const radius2 = Math.max(2, Math.hypot(edge2.x - center2.x, edge2.y - center2.y));

    // Opaque black disk so the sphere reads solid (not transparent wireframe).
    ctx.fillStyle = "#000000";
    ctx.beginPath();
    ctx.arc(center2.x, center2.y, radius2, 0, TAU);
    ctx.fill();

    // Single outline circle per planet.
    ctx.strokeStyle = rgba(planetColor, 0.95);
    ctx.lineWidth = 1.2 + clamp(4 / (center2.z * 0.011 + 1), 0.3, 1.8);
    ctx.beginPath();
    ctx.arc(center2.x, center2.y, radius2, 0, TAU);
    ctx.stroke();
  }

  const star2 = project(starPos, camera);
  if (star2) {
    ctx.fillStyle = rgba(colors.white, 0.98);
    ctx.font = `${11 * devicePixelRatio}px "IBM Plex Mono", monospace`;
    ctx.fillText("SUN", star2.x + 14, star2.y - 12);
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
  drawObject(state.ship.pos, state.ship.yaw, 36, verts, edges, colors.white, camera, { alpha: 0.95, baseWidth: 1 });
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
  if (!state.eva.deployed) return;
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
  if (state.mode === "SHIP" || state.mode === "SHIP_INT") return state.ship;
  if (state.mode === "EVA") return state.eva.deployed ? state.eva : state.module;
  return state.module;
}

function cameraRigForMode(mode) {
  if (mode === "SHIP") {
    return { back: 380, side: 48, height: 180, fovScale: 0.76, posLerp: 0.18 };
  }
  if (mode === "SHIP_INT") {
    return { back: 70, side: 10, height: 24, fovScale: 0.98, posLerp: 0.22 };
  }
  if (mode === "EVA" && state.eva.deployed) {
    return { back: 200, side: 24, height: 105, fovScale: 0.9, posLerp: 0.2 };
  }
  return { back: 300, side: 36, height: 130, fovScale: 0.82, posLerp: 0.2 };
}

function drawShipInteriorOverlay() {
  if (state.mode !== "SHIP_INT") return;
  const w = canvas.width;
  const h = canvas.height;
  const panelW = Math.min(w * 0.45, 430);
  const panelH = Math.min(h * 0.52, 360);
  const x = w - panelW - 22;
  const y = h - panelH - 22;

  ctx.fillStyle = "rgba(0, 0, 0, 0.45)";
  ctx.fillRect(x, y, panelW, panelH);
  ctx.strokeStyle = rgba(colors.white, 0.35);
  ctx.lineWidth = 1;
  ctx.strokeRect(x, y, panelW, panelH);

  const nodes = [
    { k: "reactor", label: "REACTOR", px: x + panelW * 0.22, py: y + panelH * 0.26 },
    { k: "comms", label: "COMMS", px: x + panelW * 0.74, py: y + panelH * 0.24 },
    { k: "thermal", label: "THERMAL", px: x + panelW * 0.24, py: y + panelH * 0.74 },
    { k: "propulsion", label: "PROPULSION", px: x + panelW * 0.74, py: y + panelH * 0.72 },
  ];

  ctx.font = `${11 * devicePixelRatio}px "IBM Plex Mono", monospace`;
  for (let i = 0; i < nodes.length; i++) {
    const a = nodes[i];
    const b = nodes[(i + 1) % nodes.length];
    const av = state.ship.systems[a.k] / 100;
    ctx.strokeStyle = rgba(colors.green, 0.22 + av * 0.65);
    ctx.lineWidth = 1.2 + av * 1.4;
    ctx.beginPath();
    ctx.moveTo(a.px, a.py);
    ctx.lineTo(b.px, b.py);
    ctx.stroke();
  }

  for (const n of nodes) {
    const v = state.ship.systems[n.k];
    const danger = n.k === "thermal" && v > 75;
    const c = danger ? colors.red : colors.white;
    ctx.strokeStyle = rgba(c, 0.9);
    ctx.lineWidth = 1.6;
    ctx.beginPath();
    ctx.arc(n.px, n.py, 16, 0, TAU);
    ctx.stroke();
    ctx.fillStyle = rgba(c, 0.9);
    ctx.fillText(`${n.label} ${v.toFixed(0)}%`, n.px + 20, n.py + 4);
  }

  ctx.fillStyle = rgba(colors.blue, 0.9);
  ctx.fillText(`SHIP INTERNAL (${state.ship.navState}) // U/J reactor  I/K comms  O/L thermal  [ / ] cruise speed  M nav mode  X detach/dock`, x + 14, y + panelH - 16);
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
  const camYaw = actor.yaw + state.mouseLook.yawOffset;
  const forward = rotateY(vec3(0, 0, -1), camYaw);
  const right = rotateY(vec3(1, 0, 0), camYaw);
  const camHeight = rig.height + state.mouseLook.pitchOffset * 180;
  const desiredPos = add(
    actor.pos,
    add(add(mul(forward, -rig.back), mul(right, rig.side)), vec3(0, camHeight, 0))
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
  drawSolarSystem(camera);
  drawAnchors(camera);
  drawSignalSphere(camera);
  drawDebris(camera);
  drawShip(camera);
  drawObjective(camera);
  drawModule(camera);
  drawEVA(camera);
  drawTether(camera);
  if (state.mode === "MODULE") drawTrajectory(camera);
  drawShipInteriorOverlay();

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
  const cruisePct = ((state.ship.cruiseSpeed - 30) / (4000 - 30)) * 100;
  hud.phase.textContent = `PHASE   ${phaseLabel(state.phase)}  |  Seed ${state.seed}`;
  hud.mode.textContent = `MODE    ${state.mode} (3rd person)  |  Ship ${state.ship.navState}  |  Signal lag ${state.signal.lagFrames}f  |  Cruise ${len(state.ship.vel).toFixed(0)} u/s (set ${state.ship.cruiseSpeed.toFixed(0)})`;
  hud.resources.textContent = `CRUISE   [${meter(cruisePct)}] set ${state.ship.cruiseSpeed.toFixed(0)} / actual ${len(state.ship.vel).toFixed(0)}\nSHIP FUEL [${meter(state.ship.fuel)}] ${state.ship.fuel.toFixed(0)}\nMODULE   [${meter(state.module.fuel)}] ${state.module.fuel.toFixed(0)}\nEVA O2   [${meter(state.eva.oxygen)}] ${state.eva.oxygen.toFixed(0)}\nSIGNAL   [${meter(state.signal.quality * 100)}] ${(state.signal.quality * 100).toFixed(0)}%\nDAMAGE   [${meter(100 - state.module.damage)}] ${state.module.damage.toFixed(0)}%`;
  hud.status.textContent = `STATUS  ${state.message} | Explorer ${state.module.attached ? "ATTACHED" : "DETACHED"} | Ship traveled ${state.ship.distanceTraveled.toFixed(0)} u`;
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
