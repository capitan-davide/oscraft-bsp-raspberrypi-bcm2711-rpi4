#!/usr/bin/env bash
# Download one file of the Raspberry Pi firmware from a pinned release and
# check its SHA-256.
#
# Usage: fetch-firmware.sh <name> <output>
#   e.g. fetch-firmware.sh start4.elf build/rpi4/fw/start4.elf
set -euo pipefail

FIRMWARE_TAG=1.20260915

name=${1:?usage: fetch-firmware.sh <name> <output>}
out=${2:?usage: fetch-firmware.sh <name> <output>}

case $name in
    bcm2711-rpi-4-b.dtb) sha256=75761b73c284e26623e4d1624bff13e67bce2ae620880efd81d6571a3739fcfb ;;
    start4.elf)          sha256=560ead37044d849325755d3684ac1f74654b3ce6450604a42df6dd832cda9ede ;;
    fixup4.dat)          sha256=d181d5055daebac22513d6904fa9ce21d1e99b852cc391af095ade6f0b45c400 ;;
    LICENCE.broadcom)    sha256=c7283ff51f863d93a275c66e3b4cb08021a5dd4d8c1e7acc47d872fbe52d3d6b ;;
    *) echo "fetch-firmware.sh: no pinned checksum for '$name'" >&2; exit 1 ;;
esac

mkdir -p "$(dirname "$out")"
curl -fsSL -o "$out.tmp" "https://raw.githubusercontent.com/raspberrypi/firmware/${FIRMWARE_TAG}/boot/${name}"
echo "${sha256}  $out.tmp" | sha256sum --check --quiet
mv "$out.tmp" "$out"
