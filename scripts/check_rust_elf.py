#!/usr/bin/env python3
"""Static sanity check only; QEMU CI is needed to establish boot."""
import struct
import subprocess
import sys
from pathlib import Path

kernel = Path(sys.argv[1])
data = kernel.read_bytes()
assert data[:4] == b"\x7fELF", "missing ELF magic"
assert data[4:6] == b"\x02\x01", "expected 64-bit little-endian ELF"
assert struct.unpack_from("<H", data, 16)[0] == 2, "expected executable ELF"
assert struct.unpack_from("<H", data, 18)[0] == 62, "expected x86-64 ELF"
symbols = subprocess.check_output(["nm", str(kernel)], text=True)
assert " kernel_entry" in symbols, "missing exported kernel entry"
segments = subprocess.check_output(["readelf", "-l", str(kernel)], text=True)
assert "LOAD" in segments, "missing loadable segments"
for marker in (
    b"EMBERS: Rust no_std",
    b"READY: Embers Rust no_std entry reached.",
    struct.pack("<QQQ", 0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 6),
):
    assert marker in data, f"missing kernel marker: {marker!r}"
print("PASS: Rust freestanding ELF layout (not a boot result)")
