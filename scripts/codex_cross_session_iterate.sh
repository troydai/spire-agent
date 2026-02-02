#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DONE_FILE="${REPO_ROOT}/.agent_iteration_done"

usage() {
  cat <<'EOF'
Usage: scripts/codex_cross_session_iterate.sh <path_to_manifest>
EOF
}

if [[ "${#}" -ne 1 ]]; then
  usage >&2
  exit 1
fi

MANIFEST_FILE="${1}"

while true; do
  if [[ ! -f "${MANIFEST_FILE}" ]]; then
    echo "manifest file not found: ${MANIFEST_FILE}" >&2
    exit 1
  fi

  if [[ -f "${DONE_FILE}" ]]; then
    echo "done marker already present: ${DONE_FILE}"
    exit 0
  fi

  echo "starting codex iteration"
  {
    cat <<EOF
Use the cross-session-iterative skill with the task manifest at:
${MANIFEST_FILE}

Task manifest contents:

EOF
    cat "${MANIFEST_FILE}"
  } | codex exec -s danger-full-access -c ask_for_approval="never" -C "${REPO_ROOT}" -

  if [[ -f "${DONE_FILE}" ]]; then
    echo "done marker created: ${DONE_FILE}"
    break
  fi

  echo "no done marker yet; continuing"
  sleep 1
done
