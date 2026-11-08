#!/usr/bin/env python3
import struct
import subprocess
import sys
from pathlib import Path
p = Path(sys.argv[1])
data = p.read_bytes()
assert data[:4] == b'\x7fELF', 'ELF magic missing'
assert data[4] == 2 and data[5] == 1, 'expected 64-bit little-endian ELF'
assert struct.unpack_from('<H', data, 18)[0] == 62, 'expected x86-64 ELF'
header = subprocess.check_output(['readelf', '-h', '-l', str(p)], text=True)
assert 'kernel_entry' in subprocess.check_output(['nm', str(p)], text=True)
assert 'LOAD' in header
for pattern in ('EMBERGLASS KERNEL', 'MEMORY: first owned physical frame'):
    assert pattern.encode() in data, f'missing expected kernel string: {pattern}'
print('PASS: x86-64 freestanding ELF with kernel entry, loadable segments and boot diagnostics')
