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

# 1. Clean and Setup
if os.path.exists(base_dir):
    shutil.rmtree(base_dir)

os.makedirs(gemini_tmp, exist_ok=True)
os.makedirs(trash_dir, exist_ok=True)
os.makedirs(audit_dir, exist_ok=True)

# 2. Rich Topic Generation Logic
ACTIONS = ["Debug", "Implement", "Refactor", "Research", "Optimize", "Document", "Test", "Discuss"]
TECH = ["Rust macros", "Async/Await", "Ratatui TUI", "PostgreSQL schema", "REST API", "OpenAI API", "Kubernetes pods", "Docker multi-stage", "Regex patterns", "Memory leaks"]
CONTEXT = ["for the new dashboard", "in the legacy module", "to improve performance", "with the team", "for security audit", "to reduce latency", "using standard library"]

def gen_random_head():
    return f"{random.choice(ACTIONS)} {random.choice(TECH)} {random.choice(CONTEXT)}"

def gen_realistic_filename(days_ago=0, seconds_offset=0):
    dt = datetime.now() - timedelta(days=days_ago, seconds=seconds_offset)
    short_uuid = str(uuid.uuid4())[:8]
    return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

def create_random_session(project_id, days_ago=0, seconds_offset=0):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    
    file_name = gen_realistic_filename(days_ago, seconds_offset)
    file_path = os.path.join(path, file_name)
    
    head_text = gen_random_head()
    data = {"messages": [{"type": "user", "content": head_text}]}
    
    with open(file_path, "w") as f:
        json.dump(data, f)
        
    mtime = time.time() - (days_ago * 86400) - seconds_offset
    os.utime(file_path, (mtime, mtime))

# 3. Sync Static Fixtures
if os.path.exists(fixtures_src):
    print(f"Loading static fixtures from {fixtures_src}...")
    for root, dirs, files in os.walk(fixtures_src):
        for file in files:
            src_path = os.path.join(root, file)
            rel_path = os.path.relpath(src_path, fixtures_src)
            dest_path = os.path.join(gemini_tmp, rel_path)
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy2(src_path, dest_path)

# 4. Generate Varied Bulk Data
RANDOM_PROJECTS = ["oss_contribution", "startup_mvp", "personal_blog", "learning_lab", "side_hustle"]

print("Generating 100+ highly varied sessions...")
random.seed(42) # Fixed seed for stable "randomness"

for i in range(120):
    proj = random.choice(RANDOM_PROJECTS)
    days = random.randint(0, 45)
    offset = random.randint(0, 86400)
    create_random_session(proj, days, offset)

# Setup .project_root for the fixed static project
with open(os.path.join(gemini_tmp, "static_proj", ".project_root"), "w") as f:
    f.write(os.path.abspath("."))

print(f"Highly varied mixed dataset ready in '{base_dir}/'.")
