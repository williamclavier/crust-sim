import subprocess, json, os, sys

ROOT = os.path.dirname(os.path.abspath(__file__))
SERVER_PATH = os.path.join(ROOT, "target", "debug", "crust_sim_server")

# Start the compiled Rust server binary directly
proc = subprocess.Popen(
    [SERVER_PATH],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
    bufsize=1,
)

def send(cmd: str):
    """Send a command and read one JSON line back, skipping debug lines."""
    print(">>>", cmd)
    proc.stdin.write(cmd + "\n")
    proc.stdin.flush()

    while True:
        line = proc.stdout.readline()
        if not line:
            continue

        line = line.strip()

        # Non-JSON lines (DEBUG prints, banner)
        if not line.startswith("{"):
            print("SERVER:", line, file=sys.stderr)
            continue

        # JSON state line
        print("<<<", line[:120] + ("..." if len(line) > 120 else ""))
        return json.loads(line)

# ---- Test sequence ----

# Reset game
state = send("RESET 0")
print("Ally elixir:", state["ally_elixir"])
print("Ally towers:", state["ally_towers"])
print("time_left:", state["time_left"])

# Take a couple of dummy steps
state = send("STEP 0 10")
print("after STEP 0 10, time_left:", state["time_left"])

state = send("STEP 1 20")
print("after STEP 1 20, time_left:", state["time_left"])

# Exit
send("EXIT")
proc.wait()