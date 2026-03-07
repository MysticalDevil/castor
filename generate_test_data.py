import os
import json
import time
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
    {"id": "long_path_project_hash", "root": "/home/omega/Work/Clients/AcmeCorp/ProjectX/ModuleY/SubModuleZ", "name": "Deep Project"},
    {"id": "no_root_project_hash", "root": None, "name": "Orphaned Project"}
]

def create_session(project_id, session_id, head_text, days_ago=0, content_lines=None):
    path = os.path.join(gemini_tmp, project_id, "chats")
    os.makedirs(path, exist_ok=True)
    
    file_path = os.path.join(path, f"session-{session_id}.json")
    
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

# Generate data
for p in projects:
    p_path = os.path.join(gemini_tmp, p["id"])
    os.makedirs(p_path, exist_ok=True)
    if p["root"]:
        with open(os.path.join(p_path, ".project_root"), "w") as f:
            f.write(p["root"])

# Project: Castor Development
create_session("castor_dev_hash", "2026-03-08-01", "Implement doctor command", 0)
create_session("castor_dev_hash", "2026-03-07-02", "Fix alignment issues with CJK characters", 1)
create_session("castor_dev_hash", "2026-02-20-03", "Initial architecture discussion\nwith multiple lines", 16)

# Project: Rust Learning (Lots of small sessions for paging)
for i in range(1, 15):
    create_session("rust_learning_hash", f"lesson-{i:02d}", f"Learning Rust Lesson {i}", i)

# Project: Web Project (Rich content for 'cat')
create_session("web_project_hash", "api-design", "Design a REST API for the user module", 2, [
    ("assistant", "Sure! I suggest using Express.js with the following endpoints..."),
    ("user", "What about authentication?"),
    ("assistant", "You should use JWT for stateless authentication.")
])

# Project: Deep Project (Test path truncation)
create_session("long_path_project_hash", "deep-file", "Testing very long host paths", 5)

# Project: Orphaned (Test fallback display)
create_session("no_root_project_hash", "orphan-1", "A session without a known host", 40) # Old session for pruning

# Simulated Trash
trash_p = os.path.join(trash_dir, "rust_learning_hash")
os.makedirs(trash_p, exist_ok=True)
with open(os.path.join(trash_p, "old-garbage.json"), "w") as f:
    json.dump({"messages": []}, f)

print("Rich test data generated successfully in 'test_data/'.")
