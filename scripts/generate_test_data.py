import os
import json
import time
import uuid
import random
import shutil
import sys
from datetime import datetime, timedelta, timezone

def generate(count=120, huge_files=1):
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

    def gen_realistic_filename(days_ago=0, seconds_offset=0):
        dt = datetime.now(timezone.utc) - timedelta(days=days_ago, seconds=seconds_offset)
        short_uuid = str(uuid.uuid4())[:8]
        return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

    def create_session(project_id, days_ago=0, is_huge=False):
        path = os.path.join(gemini_tmp, project_id, "chats")
        os.makedirs(path, exist_ok=True)
        file_name = gen_realistic_filename(days_ago)
        file_path = os.path.join(path, file_name)
        
        head = gen_random_head()
        data = {"messages": [{"type": "user", "content": head}]}
        with open(file_path, "w") as f:
            json.dump(data, f)
            if is_huge:
                f.write(" " * 1024 * 1024 * 51)
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

    # 4. Bulk Data
    random.seed(42)
    BULK_PROJECTS = ["p_alpha", "p_beta", "p_gamma", "p_delta", "p_epsilon"]
    print(f"Generating {count} varied sessions...")
    for i in range(count):
        proj = random.choice(BULK_PROJECTS)
        create_session(proj, random.randint(0, 100))

    # 5. Anomalies
    for i in range(huge_files):
        create_session("anomalies", is_huge=True)

    # 6. Sandbox Setup
    all_valid_projects = BULK_PROJECTS + ["standard_sessions", "multilingual", "edge_cases", "security_audit", "anomalies"]
    for proj in all_valid_projects:
        os.makedirs(os.path.join(fake_projects_dir, proj), exist_ok=True)
        with open(os.path.join(gemini_tmp, proj, ".project_root"), "w") as f:
            f.write(os.path.join(fake_projects_dir, proj))

    print(f"Stress dataset ready in '{base_dir}/'. Total: ~{count + 15}")

if __name__ == "__main__":
    count = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    generate(count)
