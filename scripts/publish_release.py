import json
import urllib.request
import subprocess
import os

p = subprocess.Popen(["git", "credential", "fill"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
out, _ = p.communicate("protocol=https\nhost=github.com\n\n")
token = None
for line in out.splitlines():
    if line.startswith("password="):
        token = line.split("=", 1)[1].strip()

if not token:
    print("Error: Could not retrieve token")
    exit(1)

with open(r"d:\CE\scrdeck\changelogs\v1.0.0.md", "r", encoding="utf-8") as f:
    body_text = f.read()

# Check if release exists
check_req = urllib.request.Request(
    "https://api.github.com/repos/ronnanakibu/scrdeck/releases/tags/V1",
    headers={
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github.v3+json",
        "User-Agent": "ScrDeck-Agent"
    }
)

release_data = None
try:
    with urllib.request.urlopen(check_req) as resp:
        release_data = json.loads(resp.read().decode("utf-8"))
        print("Existing release found, ID:", release_data["id"])
except urllib.error.HTTPError as e:
    if e.code == 404:
        print("Release does not exist, creating...")
    else:
        print("Check failed:", e.code, e.read().decode("utf-8"))

if not release_data:
    url = "https://api.github.com/repos/ronnanakibu/scrdeck/releases"
    payload = {
        "tag_name": "V1",
        "name": "ScrDeck V1 - Initial Public Release",
        "body": body_text,
        "draft": False,
        "prerelease": False
    }
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github.v3+json",
            "User-Agent": "ScrDeck-Agent",
            "Content-Type": "application/json"
        },
        method="POST"
    )
    with urllib.request.urlopen(req) as resp:
        release_data = json.loads(resp.read().decode("utf-8"))
        print("Created release ID:", release_data["id"])

upload_url = release_data["upload_url"].split("{")[0]

files_to_upload = [
    (r"d:\CE\scrdeck\release\ScrDeck-v1.0.0-Setup.exe", "ScrDeck-v1.0.0-Setup.exe", "application/vnd.microsoft.portable-executable"),
    (r"d:\CE\scrdeck\release\ScrDeck-v1.0.0-Portable.zip", "ScrDeck-v1.0.0-Portable.zip", "application/zip"),
    (r"d:\CE\scrdeck\release\scrdeck.exe", "scrdeck.exe", "application/vnd.microsoft.portable-executable"),
    (r"d:\CE\scrdeck\release\WebView2Loader.dll", "WebView2Loader.dll", "application/x-msdownload")
]

# Check existing assets
existing_assets = {a["name"]: a["id"] for a in release_data.get("assets", [])}

for file_path, name, content_type in files_to_upload:
    if name in existing_assets:
        print(f"Asset {name} already exists (ID: {existing_assets[name]}), deleting old asset...")
        del_req = urllib.request.Request(
            f"https://api.github.com/repos/ronnanakibu/scrdeck/releases/assets/{existing_assets[name]}",
            headers={
                "Authorization": f"Bearer {token}",
                "Accept": "application/vnd.github.v3+json",
                "User-Agent": "ScrDeck-Agent"
            },
            method="DELETE"
        )
        with urllib.request.urlopen(del_req) as _:
            pass

    print(f"Uploading {name} ({os.path.getsize(file_path)} bytes)...")
    with open(file_path, "rb") as f:
        file_bytes = f.read()

    upload_req = urllib.request.Request(
        f"{upload_url}?name={name}",
        data=file_bytes,
        headers={
            "Authorization": f"Bearer {token}",
            "Accept": "application/vnd.github.v3+json",
            "User-Agent": "ScrDeck-Agent",
            "Content-Type": content_type
        },
        method="POST"
    )
    with urllib.request.urlopen(upload_req) as resp:
        res = json.loads(resp.read().decode("utf-8"))
        print(f"Uploaded {name} successfully! Download URL: {res['browser_download_url']}")
