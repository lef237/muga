#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
usage:
  scripts/privacy-guard.sh pre-commit
  scripts/privacy-guard.sh commit-msg <message-file>
  scripts/privacy-guard.sh pre-push <remote-name> <remote-url>

The guard reads forbidden literal strings from these local-only files:
  - $MUGA_PRIVACY_DENYLIST, when set
  - .git/info/privacy-denylist
  - .privacy-denylist

Blank lines and lines beginning with # are ignored.
USAGE
}

mode="${1:-}"
if [[ -z "$mode" ]]; then
  usage
  exit 2
fi
shift

repo_root="$(git rev-parse --show-toplevel)"
git_denylist="$(git rev-parse --git-path info/privacy-denylist)"

denylist_files=()
if [[ -n "${MUGA_PRIVACY_DENYLIST:-}" ]]; then
  denylist_files+=("$MUGA_PRIVACY_DENYLIST")
fi
denylist_files+=("$git_denylist" "$repo_root/.privacy-denylist")

patterns=()
for denylist_file in "${denylist_files[@]}"; do
  [[ -f "$denylist_file" ]] || continue
  while IFS= read -r line || [[ -n "$line" ]]; do
    line="${line%$'\r'}"
    [[ -z "$line" ]] && continue
    [[ "$line" == \#* ]] && continue
    patterns+=("$line")
  done < "$denylist_file"
done

if [[ "${#patterns[@]}" -eq 0 ]]; then
  cat >&2 <<EOF
privacy guard: no forbidden strings are configured.

Create a local-only denylist and add one literal string per line:
  $git_denylist

You can also use:
  $repo_root/.privacy-denylist
EOF
  exit 1
fi

grep_args=(-I -n -F)
for pattern in "${patterns[@]}"; do
  grep_args+=(-e "$pattern")
done

capture_git_grep() {
  local output
  local status

  set +e
  output="$(git grep "${grep_args[@]}" "$@" 2>&1)"
  status=$?
  set -e

  case "$status" in
    0)
      redact_grep_output "$output"
      return 0
      ;;
    1)
      return 1
      ;;
    *)
      printf 'privacy guard: git grep failed:\n%s\n' "$output" >&2
      exit "$status"
      ;;
  esac
}

capture_file_grep() {
  local file="$1"
  local output
  local status

  set +e
  output="$(grep "${grep_args[@]}" -- "$file" 2>&1)"
  status=$?
  set -e

  case "$status" in
    0)
      redact_grep_output "$output"
      return 0
      ;;
    1)
      return 1
      ;;
    *)
      printf 'privacy guard: grep failed:\n%s\n' "$output" >&2
      exit "$status"
      ;;
  esac
}

redact_grep_output() {
  local output="$1"

  printf '%s\n' "$output" | awk -F: '
    $1 ~ /^[0-9a-f]{40}$/ && NF >= 3 {
      print $1 ":" $2 ":" $3 ":<redacted>"
      next
    }
    $1 ~ /^[0-9]+$/ && NF >= 2 {
      print $1 ":<redacted>"
      next
    }
    $2 ~ /^[0-9]+$/ && NF >= 3 {
      print $1 ":" $2 ":<redacted>"
      next
    }
    NF >= 2 {
      print $1 ":<redacted>"
      next
    }
    {
      print "<redacted>"
    }
  '
}

capture_text_matches() {
  local label="$1"
  local text="$2"
  local line
  local line_number=0
  local pattern
  local found=1

  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    for pattern in "${patterns[@]}"; do
      if [[ "$line" == *"$pattern"* ]]; then
        printf '%s:%s:<redacted>\n' "$label" "$line_number"
        found=0
        break
      fi
    done
  done <<< "$text"

  return "$found"
}

report_matches() {
  local header="$1"
  local matches="$2"

  printf '%s\n' "$header" >&2
  printf '%s\n' "$matches" >&2
  printf '\nprivacy guard: blocked because forbidden text was found.\n' >&2
}

scan_index() {
  local matches

  if matches="$(capture_git_grep --cached -- .)"; then
    report_matches "privacy guard: forbidden text found in staged content:" "$matches"
    exit 1
  fi
}

scan_commit_message_file() {
  local message_file="$1"
  local matches

  if matches="$(capture_file_grep "$message_file")"; then
    report_matches "privacy guard: forbidden text found in commit message:" "$matches"
    exit 1
  fi
}

scan_push() {
  local remote_name="${1:-origin}"
  local _remote_url="${2:-}"
  local zero_oid="0000000000000000000000000000000000000000"
  local local_ref
  local local_oid
  local remote_ref
  local remote_oid
  local found=0
  local rev_list_status
  local commits
  local commit
  local matches
  local commit_record

  while read -r local_ref local_oid remote_ref remote_oid; do
    [[ -n "${local_ref:-}" ]] || continue
    [[ "$local_oid" != "$zero_oid" ]] || continue

    set +e
    if [[ "$remote_oid" == "$zero_oid" ]]; then
      commits="$(git rev-list "$local_oid" --not "--remotes=$remote_name" 2>&1)"
    else
      commits="$(git rev-list "$remote_oid..$local_oid" 2>&1)"
    fi
    rev_list_status=$?
    set -e

    if [[ "$rev_list_status" -ne 0 ]]; then
      printf 'privacy guard: git rev-list failed for %s -> %s:\n%s\n' \
        "$local_ref" "$remote_ref" "$commits" >&2
      exit "$rev_list_status"
    fi

    while IFS= read -r commit; do
      [[ -n "$commit" ]] || continue
      if matches="$(capture_git_grep "$commit" -- .)"; then
        report_matches "privacy guard: forbidden text found in commit $commit:" "$matches"
        found=1
      fi
      commit_record="$(git log -1 --format='%H%n%an%n%ae%n%cn%n%ce%n%B' "$commit")"
      if matches="$(capture_text_matches "$commit:metadata" "$commit_record")"; then
        report_matches "privacy guard: forbidden text found in commit metadata $commit:" "$matches"
        found=1
      fi
    done <<< "$commits"
  done

  if [[ "$found" -ne 0 ]]; then
    exit 1
  fi
}

case "$mode" in
  pre-commit)
    scan_index
    ;;
  commit-msg)
    scan_commit_message_file "$@"
    ;;
  pre-push)
    scan_push "$@"
    ;;
  *)
    usage
    exit 2
    ;;
esac
