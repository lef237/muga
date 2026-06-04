#!/usr/bin/env bash
set -euo pipefail

run_publish_dry_run=0
gate_tmp="${HOME}/tmp/muga-v1-release-gate.$$"

for arg in "$@"; do
  case "$arg" in
    --with-publish-dry-run)
      run_publish_dry_run=1
      ;;
    *)
      echo "unknown argument: $arg" >&2
      echo "usage: scripts/v1-release-gate.sh [--with-publish-dry-run]" >&2
      exit 2
      ;;
  esac
done

cleanup_gate_tmp() {
  if [[ -d "$gate_tmp" ]] && command -v trash >/dev/null 2>&1; then
    trash "$gate_tmp" >/dev/null 2>&1 || true
  fi
}
trap cleanup_gate_tmp EXIT

first_line() {
  printf '%s\n' "$1" | sed -n '1p'
}

second_line() {
  printf '%s\n' "$1" | sed -n '2p'
}

cargo fmt --check
scripts/clippy-check.sh
cargo test --locked
cargo build --locked

mkdir -p "$gate_tmp"

target/debug/muga check samples/println_sum.muga
target/debug/muga samples/println_sum.muga
target/debug/muga check samples/packages/app/std_process/main.muga
target/debug/muga samples/packages/app/std_process/main.muga
target/debug/muga build samples/packages/app/std_process/main.muga
target/debug/muga check --built samples/packages/app/std_process/main.muga
target/debug/muga run --built samples/packages/app/std_process/main.muga
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/std-process-bundle" --program std-process samples/projects/process_app/src/main/main.muga
target/debug/muga run-app-bundle "$gate_tmp/std-process-bundle"
target/debug/muga build samples/packages/app/artifact_facade/main.muga
target/debug/muga check --built samples/packages/app/artifact_facade/main.muga
target/debug/muga run --built samples/packages/app/artifact_facade/main.muga
target/debug/muga api-diff --old-artifact-root samples/packages/app/artifact_facade/.muga/build --new-artifact-root samples/packages/app/artifact_facade/.muga/build --package app::artifact_facade --fail-on breaking

package_archive_output="$(target/debug/muga emit-package-archive --archive-root "$gate_tmp/package-archives" samples/projects/local_path_shared/src/logging/main.muga)"
package_archive_path="$(first_line "$package_archive_output")"
package_archive_hash="$(second_line "$package_archive_output")"
target/debug/muga verify-package-archive "$package_archive_path"
package_archive_renamed="$gate_tmp/package-archives/release-gate.mgp"
cp "$package_archive_path" "$package_archive_renamed"
target/debug/muga verify-package-archive --expected-hash "$package_archive_hash" "$package_archive_renamed"
target/debug/muga unpack-package-archive --expected-hash "$package_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-package" "$package_archive_renamed"
target/debug/muga check "$gate_tmp/renamed-unpacked-package/src/logging/main.muga"

cp -R samples/projects/my_service "$gate_tmp/my_service"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/app-bundle" --program release-gate "$gate_tmp/my_service/src/main/main.muga"
app_archive_output="$(target/debug/muga emit-app-archive --archive-root "$gate_tmp/app-archives" --program release-gate "$gate_tmp/app-bundle")"
app_archive_path="$(first_line "$app_archive_output")"
app_archive_hash="$(second_line "$app_archive_output")"
target/debug/muga verify-app-archive "$app_archive_path"
app_archive_renamed="$gate_tmp/app-archives/release-gate.mga"
cp "$app_archive_path" "$app_archive_renamed"
target/debug/muga verify-app-archive --expected-hash "$app_archive_hash" "$app_archive_renamed"
target/debug/muga unpack-app-archive --expected-hash "$app_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-app" "$app_archive_renamed"
target/debug/muga unpack-app-archive --output-dir "$gate_tmp/unpacked-app" "$app_archive_path"
target/debug/muga run-app-bundle "$gate_tmp/unpacked-app"
target/debug/muga install-app --output-dir "$gate_tmp/installed-bin" --program release-gate "$gate_tmp/unpacked-app"
target/debug/muga list-installed-apps --output-dir "$gate_tmp/installed-bin"
MUGA_BIN="$PWD/target/debug/muga" "$gate_tmp/installed-bin/release-gate"
target/debug/muga uninstall-app --output-dir "$gate_tmp/installed-bin" --program release-gate
cp -R samples/projects/resource_export "$gate_tmp/resource_export"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/resource-export-bundle" --program resource-export "$gate_tmp/resource_export/src/main/main.muga"
target/debug/muga run-app-bundle "$gate_tmp/resource-export-bundle" -- "$gate_tmp/resource-export-payload.bin"

cargo package --locked --allow-dirty --offline --list
cargo package --locked --allow-dirty --offline

if [[ "$run_publish_dry_run" == "1" ]]; then
  cargo publish --dry-run --locked
fi
