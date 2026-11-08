#!/usr/bin/env bash
set -euo pipefail
# External bootloader. Verify release availability and checksum before relying on this.
LIMINE_TAG="v12.9.6"
for tool in curl tar make xorriso sha256sum; do
  command -v "$tool" >/dev/null || { echo "Missing dependency: $tool" >&2; exit 1; }
done
if [ ! -d limine-binary ]; then
  curl -fL --retry 2 -o build/limine-binary.tar.gz "https://github.com/Limine-Bootloader/Limine/releases/download/${LIMINE_TAG}/limine-binary.tar.gz"
  printf "%s  %s\n" "b38aabb042b0c0f9f562b7741be3c1204838d7bbacfa78bb411c674bd5b984ee" "build/limine-binary.tar.gz" | sha256sum --check
  tar -xzf build/limine-binary.tar.gz
fi
make -C limine-binary
mkdir -p build/iso_root/boot/limine build/iso_root/EFI/BOOT
cp build/emberglass.elf build/iso_root/boot/
cp limine.conf limine-binary/limine-bios.sys limine-binary/limine-bios-cd.bin limine-binary/limine-uefi-cd.bin build/iso_root/boot/limine/
cp limine-binary/BOOTX64.EFI build/iso_root/EFI/BOOT/
xorriso -as mkisofs -R -r -J -b boot/limine/limine-bios-cd.bin -no-emul-boot -boot-load-size 4 -boot-info-table --efi-boot boot/limine/limine-uefi-cd.bin -efi-boot-part --efi-boot-image --protective-msdos-label build/iso_root -o build/emberglass.iso
./limine-binary/limine bios-install build/emberglass.iso
echo "ISO ready: build/emberglass.iso"
