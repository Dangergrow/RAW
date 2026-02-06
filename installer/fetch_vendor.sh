#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
vendor_dir="${root_dir}/vendor"
config_offline="${root_dir}/.cargo/config.offline.toml"
config_active="${root_dir}/.cargo/config.toml"

if [[ -d "${vendor_dir}" && -n "$(ls -A "${vendor_dir}")" ]]; then
  echo "Vendor directory already present."
else
  if [[ -n "${PLUS_VENDOR_URL:-}" ]]; then
    echo "Downloading vendor archive from ${PLUS_VENDOR_URL}..."
    curl -L "${PLUS_VENDOR_URL}" -o /tmp/vendor.tar.zst
    mkdir -p "${vendor_dir}"
    tar --zstd -xvf /tmp/vendor.tar.zst -C "${root_dir}"
  else
    echo "Vendor archive not set. Running cargo vendor..."
    cargo vendor "${vendor_dir}" > /tmp/vendor-config.toml
  fi
fi

if [[ ! -d "${vendor_dir}" || -z "$(ls -A "${vendor_dir}")" ]]; then
  echo "Vendor directory is empty. Cannot enable offline mode."
  exit 1
fi

cp "${config_offline}" "${config_active}"
echo "Offline config enabled at ${config_active}."
