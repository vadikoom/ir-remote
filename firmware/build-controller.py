import json
import os
from pathlib import Path
import re
import subprocess
import tempfile


if not re.fullmatch(r"[0-9a-fA-F]{64}", os.environ["FIRMWARE_SHARED_SECRET"]):
    raise SystemExit("FIRMWARE_SHARED_SECRET must contain exactly 64 hexadecimal characters")

with tempfile.TemporaryDirectory() as directory:
    header = Path(directory) / "environment.h"
    header.write_text("\n".join(
        "#define " + key + " " + json.dumps(os.environ[key])
        for key in ["FIRMWARE_WIFI_SSID", "FIRMWARE_WIFI_PASS", "FIRMWARE_REMOTE_HOST", "FIRMWARE_SHARED_SECRET"]
    ) + "\n")
    header.chmod(0o600)
    subprocess.run([
        "arduino-cli", "compile", "--fqbn", os.environ["CONTROLLER_FQBN"],
        "--build-property", "compiler.cpp.extra_flags=-include " + str(header),
        "--output-dir", "/app/bin/controller", "/app/controller",
    ], check=True)
