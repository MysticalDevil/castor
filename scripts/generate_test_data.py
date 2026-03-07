import os
import json
import time
import uuid
import random
import shutil
from datetime import datetime, timedelta

# Paths
base_dir = "test_data"
gemini_tmp = os.path.join(base_dir, "gemini_tmp")
trash_dir = os.path.join(base_dir, "trash")
audit_dir = os.path.join(base_dir, "audit")
fixtures_src = "tests/common/fixtures"

# 1. Reset Environment
if os.path.exists(base_dir):
    shutil.rmtree(base_dir)

os.makedirs(gemini_tmp, exist_ok=True)
os.makedirs(trash_dir, exist_ok=True)
os.makedirs(audit_dir, exist_ok=True)

# 2. Sync Static Fixtures (The "Grounded" part)
if os.path.exists(fixtures_src):
    print(f"Loading static fixtures from {fixtures_src}...")
    for root, dirs, files in os.walk(fixtures_src):
        for file in files:
            src_path = os.path.join(root, file)
            rel_path = os.path.relpath(src_path, fixtures_src)
            dest_path = os.path.join(gemini_tmp, rel_path)
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy2(src_path, dest_path)

# 3. Dynamic Generation Templates
ACTIONS = ["Debug", "Refactor", "Optimize", "Audit", "Discuss", "Architect"]
TECH = ["Rust generics", "Tokio Runtime", "WebGPU shaders", "Redis clusters", "CI/CD pipelines", "Memory profiling"]
SCENARIOS = ["in production", "for the upcoming launch", "with legacy dependencies", "to reduce costs", "across multiple regions"]

def gen_realistic_filename(days_ago=0, seconds_offset=0, future=False):
    dt = datetime.now() - timedelta(days=days_ago, seconds=seconds_offset)
    if future:
        dt += timedelta(days=400)
    short_uuid = str(uuid.uuid4())[:8]
    return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

def create_bulk_session(project_id, days_ago=0, is_huge=False):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    
    file_name = gen_realistic_filename(days_ago)
    file_path = os.path.join(path, file_name)
    
    head = f"{random.choice(ACTIONS)} {random.choice(TECH)} {random.choice(SCENARIOS)}"
    data = {"messages": [{"type": "user", "content": head}]}
    
    with open(file_path, "w") as f:
        json.dump(data, f)
        if is_huge:
            f.write(" " * 1024 * 1024 * 51) # 51MB to trigger RISK
            
    mtime = time.time() - (days_ago * 86400)
    os.utime(file_path, (mtime, mtime))

# 4. Generate Data
random.seed(1337)
BULK_PROJECTS = ["open_source", "work_tasks", "private_logs"]

print("Generating varied bulk data...")
# Healthy bulk
for i in range(100):
    proj = random.choice(BULK_PROJECTS)
    create_bulk_session(proj, random.randint(0, 40))

# Edge Cases via script
print("Injecting statistical anomalies...")
create_bulk_session("anomalies", is_huge=True)

# 5. Create Metadata
# Valid project roots
for proj in BULK_PROJECTS + ["standard_sessions", "multilingual", "edge_cases"]:
    with open(os.path.join(gemini_tmp, proj, ".project_root"), "w") as f:
        f.write(f"/home/omega/Projects/{proj}")

# Orphaned project root (WARN state)
os.makedirs(os.path.join(gemini_tmp, "orphaned_proj"), exist_ok=True)
with open(os.path.join(gemini_tmp, "orphaned_proj", ".project_root"), "w") as f:
    f.write("/non/existent/path/on/system")

print("Refreshing test data complete.")
