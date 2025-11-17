import subprocess, json, os, sys
import numpy as np
import gymnasium as gym
from gymnasium import spaces

ROOT = os.path.dirname(os.path.abspath(__file__))
SERVER_PATH = os.path.join(ROOT, "target", "debug", "crust_sim_server")

def _start_server():
    return subprocess.Popen(
        [SERVER_PATH],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

def _read_json_line(proc):
    while True:
        line = proc.stdout.readline()
        if not line:
            raise RuntimeError("Server closed stdout")
        line = line.strip()
        if not line:
            continue
        if line.startswith("{"):
            return json.loads(line)
        else:
            print("SERVER:", line, file=sys.stderr)

def _send(proc, cmd: str):
    proc.stdin.write(cmd + "\n")
    proc.stdin.flush()
    return _read_json_line(proc)

class CRSimEnv(gym.Env):
    metadata = {"render_modes": []}

    def __init__(self, render_mode=None):
        super().__init__()
        self.render_mode = render_mode

        # 8 hand slots, 16x9 tile grid (same as LegalMasks placeholder)
        self.action_space = spaces.MultiDiscrete([8, 16 * 9])

        # Observation: [ally_elixir, time_left,
        #               3 ally tower hp, 3 enemy tower hp]  => 8 floats
        self.observation_space = spaces.Box(
            low=0.0, high=1e4, shape=(8,), dtype=np.float32
        )

        self.proc = _start_server()
        self.last_state = None

    def _obs_from_state(self, s):
        ally_elixir = s["ally_elixir"]
        time_left = s["time_left"]

        # Sort towers by x to get consistent ordering (left, king, right)
        def hp_vec(towers):
            towers_sorted = sorted(towers, key=lambda t: t["x"])
            return [t["hp_frac"] for t in towers_sorted]

        ally_hp = hp_vec(s["ally_towers"])
        enemy_hp = hp_vec(s["enemy_towers"])

        vec = np.array(
            [ally_elixir, time_left] + ally_hp + enemy_hp, dtype=np.float32
        )
        return vec

    def reset(self, *, seed=None, options=None):
        if seed is None:
            seed = 0
        s = _send(self.proc, f"RESET {seed}")
        self.last_state = s
        obs = self._obs_from_state(s)
        info = {}
        return obs, info

    def step(self, action):
        card_idx, tile_idx = int(action[0]), int(action[1])
        s = _send(self.proc, f"STEP {card_idx} {tile_idx}")
        self.last_state = s
        obs = self._obs_from_state(s)

        # Very basic reward: damage dealt - 0.5 * damage taken (per tick)
        r = float(s["enemy_tower_hp_drop"] - 0.5 * s["ally_tower_hp_drop"])

        terminated = bool(s["win"] or s["lose"])
        truncated = False  # you could also truncate when time_left == 0
        info = {}
        return obs, r, terminated, truncated, info

    def close(self):
        if self.proc is not None:
            try:
                self.proc.stdin.write("EXIT\n")
                self.proc.stdin.flush()
                self.proc.wait(timeout=1)
            except Exception:
                pass
            self.proc = None