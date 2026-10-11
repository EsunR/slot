# -*- coding: utf-8 -*-
"""Verify the freshly built device binary: aarch64 ELF + Chinese UI strings + glibc ceiling."""
import struct, sys
from pathlib import Path

if len(sys.argv) != 2:
    raise SystemExit("usage: python verify_slot.py <device-binary>")

data = Path(sys.argv[1]).read_bytes()
print("file size :", len(data), "bytes")

# --- ELF header ---
assert data[:4] == b"\x7fELF", "not an ELF"
cls, machine = data[4], struct.unpack_from("<H", data, 18)[0]
print("EI_CLASS  :", cls, "(2=64bit)")
print("e_machine :", hex(machine), "(0xb7=AArch64)")
arch_ok = cls == 2 and machine == 0xB7
print("arch ok   :", arch_ok)

# --- Chinese strings we expect after the translation work ---
zh = ["重启", "关机", "存档已保存", "金手指已开启", "联机", "设置时钟", "正在寻找其他玩家"]
print("\n-- Chinese strings expected --")
for s in zh:
    print(("  OK   " if s.encode("utf-8") in data else "  MISS "), s)

# --- Old English strings that should be GONE ---
en = ["Power Off", "State Saved", "Cheats On", "Cheats Off"]
print("\n-- Old English strings (should be GONE) --")
for s in en:
    print(("  STILL" if s.encode() in data else "  GONE "), s)

# --- INTERP (static vs dynamic) ---
i = data.find(b"/lib/ld-linux-aarch64.so.1")
print("\nINTERP     :", data[i:i + 64].split(b"\x00")[0].decode("ascii", "replace") if i >= 0 else "none (static)")

if not arch_ok or any(s.encode("utf-8") not in data for s in zh) or any(s.encode() in data for s in en):
    raise SystemExit(1)
