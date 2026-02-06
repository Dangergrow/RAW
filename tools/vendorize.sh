#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
vendor_dir="${root_dir}/vendor"
config_offline="${root_dir}/.cargo/config.offline.toml"
config_active="${root_dir}/.cargo/config.toml"

mkdir -p "${vendor_dir}"

echo "Vendoring crates into ${vendor_dir}..."
cargo vendor "${vendor_dir}" > /tmp/vendor-config.toml

echo "Activating offline config..."
cp "${config_offline}" "${config_active}"

echo "Vendor ready. You can now build with: cargo build --offline"
