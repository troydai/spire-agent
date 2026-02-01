#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROMPT_FILE="${REPO_ROOT}/prompt.md"
DONE_FILE="${REPO_ROOT}/.codex_done"

while true; do
  if [[ ! -f "${PROMPT_FILE}" ]]; then
    echo "prompt file not found: ${PROMPT_FILE}" >&2
    exit 1
  fi

  if [[ -f "${DONE_FILE}" ]]; then
    echo "done marker already present: ${DONE_FILE}"
    exit 0
  fi

  echo "starting codex iteration"
  PROMPT_CONTENT="$(cat "${PROMPT_FILE}")"
  codex -a untrusted "${PROMPT_CONTENT}"

  if [[ -f "${DONE_FILE}" ]]; then
    echo "done marker created: ${DONE_FILE}"
    break
  fi

  echo "no done marker yet; continuing"
  sleep 1
done
