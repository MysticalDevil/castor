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

# 2. Sync Static Fixtures (The "Fixed" part)
if os.path.exists(fixtures_src):
    print(f"Loading static fixtures from {fixtures_src}...")
    for root, dirs, files in os.walk(fixtures_src):
        for file in files:
            src_path = os.path.join(root, file)
            # Maintain the same structure under gemini_tmp
            rel_path = os.path.relpath(src_path, fixtures_src)
            dest_path = os.path.join(gemini_tmp, rel_path)
            os.makedirs(os.path.dirname(dest_path), exist_ok=True)
            shutil.copy2(src_path, dest_path)

# 3. Generate Random Data (The "Massive" part)
def gen_realistic_filename(days_ago=0, seconds_offset=0):
    dt = datetime.now() - timedelta(days=days_ago, seconds=seconds_offset)
    short_uuid = str(uuid.uuid4())[:8]
    return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

def create_random_session(project_id, head_text, days_ago=0):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    file_name = gen_realistic_filename(days_ago)
    file_path = os.path.join(path, file_name)
    data = {"messages": [{"type": "user", "content": head_text}]}
    with open(file_path, "w") as f:
        json.dump(data, f)
    mtime = time.time() - (days_ago * 86400)
    os.utime(file_path, (mtime, mtime))

print("Generating 100+ random sessions for stress testing...")
for i in range(100):
    create_random_session("generated_bulk_study", f"Randomly generated study session #{i}", random.randint(0, 30))

# Create .project_root for the fixed static project
with open(os.path.join(gemini_tmp, "static_proj", ".project_root"), "w") as f:
    f.write(os.path.abspath("."))

print(f"Mixed dataset ready in '{base_dir}/'. (Static + Generated)")
