import os
import json
import time
import uuid
from datetime import datetime, timedelta

base_dir = "test_data"
gemini_tmp = os.path.join(base_dir, "gemini_tmp")
trash_dir = os.path.join(base_dir, "trash")
audit_dir = os.path.join(base_dir, "audit")

# Ensure directories exist
os.makedirs(gemini_tmp, exist_ok=True)
os.makedirs(trash_dir, exist_ok=True)
os.makedirs(audit_dir, exist_ok=True)

projects = [
    {"id": "castor_dev_hash", "root": "/home/omega/Projects/castor", "name": "Castor Development"},
    {"id": "rust_learning_hash", "root": "/home/omega/Learning/rust-basics", "name": "Rust Learning"},
    {"id": "web_project_hash", "root": "/var/www/my-app", "name": "Web Project"},
    {"id": "broken_project_hash", "root": "/non/existent/path/for/warning", "name": "Missing Host Project"},
    {"id": "no_root_project_hash", "root": None, "name": "Orphaned Project"}
]

def gen_realistic_filename(days_ago=0):
    dt = datetime.now() - timedelta(days=days_ago)
    short_uuid = str(uuid.uuid4())[:8]
    return f"session-{dt.strftime('%Y-%m-%dT%H-%M')}-{short_uuid}.json"

def create_session(project_id, head_text, days_ago=0, content_lines=None, is_corrupted=False):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    
    file_name = gen_realistic_filename(days_ago)
    file_path = os.path.join(path, file_name)
    
    if is_corrupted:
        # Create an empty file to trigger ERROR state
        with open(file_path, "w") as f:
            pass
    else:
        messages = [{"type": "user", "content": [{"text": head_text}]}]
        if content_lines:
            for role, text in content_lines:
                messages.append({"type": role, "content": [{"text": text}]})
                
        data = {"messages": messages}
        with open(file_path, "w") as f:
            json.dump(data, f)
        
    # Adjust timestamps
    mtime = time.time() - (days_ago * 86400)
    os.utime(file_path, (mtime, mtime))

# Generate Project Roots
for p in projects:
    p_path = os.path.join(gemini_tmp, p["id"])
    os.makedirs(p_path, exist_ok=True)
    if p["root"]:
        with open(os.path.join(p_path, ".project_root"), "w") as f:
            f.write(p["root"])

# Project: Castor Development (Healthy)
create_session("castor_dev_hash", "Implement doctor command", 0)
create_session("castor_dev_hash", "Fix alignment issues", 1)

# Project: Rust Learning (Lots of healthy sessions)
for i in range(1, 10):
    create_session("rust_learning_hash", f"Learning Rust Lesson {i}", i)

# Project: Broken Project (WARN state - host path doesn't exist)
create_session("broken_project_hash", "This session's host is missing", 2)

# Project: Web Project (Includes an ERROR session)
create_session("web_project_hash", "Design a REST API", 3)
create_session("web_project_hash", "CORRUPTED SESSION", 4, is_corrupted=True)

# Project: Orphaned (Untracked)
create_session("no_root_project_hash", "A session without a known host", 40)

print("More realistic and diverse test data generated successfully in 'test_data/'.")
