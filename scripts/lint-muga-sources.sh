#!/usr/bin/env bash
set -euo pipefail

muga_bin="${MUGA_BIN:-target/debug/muga}"

for file in samples/*.muga; do
  "$muga_bin" lint "$file" >/dev/null
done

for file in samples/packages/app/*/main.muga; do
  case "$file" in
    samples/packages/app/enum_private_import/main.muga | \
      samples/packages/app/enum_private_visibility/main.muga)
      continue
      ;;
  esac
  "$muga_bin" lint "$file" >/dev/null
done

for file in samples/projects/*/src/main/main.muga; do
  "$muga_bin" lint "$file" >/dev/null
done

while IFS= read -r file; do
  "$muga_bin" lint "$file" >/dev/null
done < <(find conformance/current/valid -name '*.muga' -type f | sort)

while IFS= read -r file; do
  "$muga_bin" lint "$file" >/dev/null
done < <(find conformance/current/package-artifacts -name 'main.muga' -type f | sort)

echo "Muga source lint passed"
