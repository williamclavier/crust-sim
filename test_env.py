from cr_env import CRSimEnv

env = CRSimEnv()
obs, info = env.reset()
print("Initial obs:", obs)

for t in range(3):
    action = env.action_space.sample()
    obs, r, done, trunc, info = env.step(action)
    print(f"step {t}: reward={r}, done={done}")

env.close()