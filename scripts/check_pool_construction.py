#!/usr/bin/env python3
"""Compile checks for the production frame-pool construction boundary."""
from pathlib import Path
import json
import subprocess
import tempfile

root = Path(__file__).resolve().parent.parent
memory_path = root / "rust-kernel" / "src" / "memory.rs"

with tempfile.TemporaryDirectory() as directory:
    temporary = Path(directory)
    prefix = '#[path = ' + json.dumps(str(memory_path)) + ']\nmod memory;\n'
    cases = [
        ("control", "fn main() { let _ = memory::claim_boot_pool_permit(); }", None),
        ("selector", "fn main() { let _ = memory::choose_pool(&[], 1); }", "E0425"),
        ("permit", "fn main() { let _ = memory::BootPoolPermit { _private: () }; }", "E0451"),
        ("clone", "fn main() { let p = memory::claim_boot_pool_permit().unwrap(); let _ = p.clone(); }", "E0599"),
        ("reuse", "fn main() { let p = memory::claim_boot_pool_permit().unwrap(); let _ = memory::select_boot_pool(p, &[], 1); let _ = memory::select_boot_pool(p, &[], 1); }", "E0382"),
    ]
    for name, body, expected in cases:
        source = temporary / (name + ".rs")
        source.write_text(prefix + body + "\n", encoding="utf-8")
        result = subprocess.run(
            ["rustc", "--edition=2021", "-A", "dead_code", str(source),
             "-o", str(temporary / name)],
            capture_output=True, text=True, check=False,
        )
        if expected is None:
            if result.returncode != 0:
                raise SystemExit("control compilation failed:\n" + result.stderr)
        elif result.returncode == 0 or f"error[{expected}]" not in result.stderr:
            raise SystemExit(name + " did not fail as expected:\n" + result.stderr)
        print(name + ": " + ("compiled" if expected is None else "rejected " + expected))
