import os
import json
import time
import uuid
import random
import shutil
from datetime import datetime, timedelta, timezone

# Paths
base_dir = os.path.abspath("test_data")
gemini_tmp = os.path.join(base_dir, "gemini_tmp")
trash_dir = os.path.join(base_dir, "trash")
audit_dir = os.path.join(base_dir, "audit")
fake_projects_dir = os.path.join(base_dir, "fake_projects")
fixtures_src = "tests/common/fixtures"

# 1. Reset Environment
if os.path.exists(base_dir):
    shutil.rmtree(base_dir)

os.makedirs(gemini_tmp, exist_ok=True)
os.makedirs(trash_dir, exist_ok=True)
os.makedirs(audit_dir, exist_ok=True)
os.makedirs(fake_projects_dir, exist_ok=True)

# 2. Dynamic Generation Templates
ACTIONS = ["Debug", "Refactor", "Optimize", "Audit", "Discuss", "Architect"]
TECH = ["Rust generics", "Tokio Runtime", "WebGPU shaders", "Redis clusters", "CI/CD pipelines", "Memory profiling"]
SCENARIOS = ["in production", "for the upcoming launch", "with legacy dependencies", "to reduce costs", "across multiple regions"]

def gen_random_head():
    return f"{random.choice(ACTIONS)} {random.choice(TECH)} {random.choice(SCENARIOS)}"

def gen_realistic_filename(days_ago=0, seconds_offset=0, future=False):
    dt = datetime.now(timezone.utc) - timedelta(days=days_ago, seconds=seconds_offset)
    if future:
        dt += timedelta(days=400)
    short_uuid = str(uuid.uuid4())[:8]
    return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

def create_session(project_id, days_ago=0, 
                   is_huge=False, 
                   is_future=False, 
                   is_malformed=False,
                   is_invalid_pattern=False):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    
    if is_invalid_pattern:
        file_name = f"random_file_{uuid.uuid4().hex[:8]}.json"
    else:
        file_name = gen_realistic_filename(days_ago, future=is_future)
        
    file_path = os.path.join(path, file_name)
    
    if is_malformed:
        with open(file_path, "w") as f:
            f.write("{ incomplete: [") # Structural Error
    else:
        head = gen_random_head()
        data = {"messages": [{"type": "user", "content": head}]}
        with open(file_path, "w") as f:
            json.dump(data, f)
            if is_huge:
                f.write(" " * 1024 * 1024 * 51) # Statistical Risk
            
    mtime = time.time() - (days_ago * 86400)
    os.utime(file_path, (mtime, mtime))

# 3. Load Static Fixtures
if os.path.exists(fixtures_src):
    print(f"Loading static fixtures from {fixtures_src}...")
    for root, dirs, files in os.walk(fixtures_src):
        for file in files:
            src_path = os.path.join(root, file)
            rel_path = os.path.relpath(src_path, fixtures_src)
            dest_path = os.path.join(gemini_tmp, rel_path)
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy2(src_path, dest_path)

# 4. Generate Varied Bulk Data (Mostly OK)
random.seed(42)
BULK_PROJECTS = ["open_source", "business_logic", "personal_notes"]

print("Generating 100+ varied sessions...")
for i in range(100):
    proj = random.choice(BULK_PROJECTS)
    create_session(proj, random.randint(0, 45))

# 5. Inject a few Randomized Anomalies (to mix with static ones)
print("Injecting sparse anomalies into generated data...")
create_session("business_logic", is_malformed=True) # 1 randomized ERROR
create_session("open_source", is_invalid_pattern=True) # 1 randomized RISK
create_session("personal_notes", is_huge=True) # 1 randomized huge RISK

# 6. Setup Host Metadata & Sandbox Folders
# These projects will be OK
valid_project_names = BULK_PROJECTS + ["standard_sessions", "multilingual", "edge_cases", "security_audit"]
for proj in valid_project_names:
    host_path = os.path.join(fake_projects_dir, proj)
    os.makedirs(host_path, exist_ok=True)
    with open(os.path.join(gemini_tmp, proj, ".project_root"), "w") as f:
        f.write(host_path)

# These projects will be WARN (Host missing)
os.makedirs(os.path.join(gemini_tmp, "abandoned_project"), exist_ok=True)
with open(os.path.join(gemini_tmp, "abandoned_project", ".project_root"), "w") as f:
    f.write("/tmp/some/deleted/folder/12345")
create_session("abandoned_project", days_ago=10)

print("Test data generation complete. Dataset contains mixed OK/WARN/ERROR/RISK states.")
