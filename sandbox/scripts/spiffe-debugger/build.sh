#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
SANDBOX_DIR="${SANDBOX_DIR:-${ROOT_DIR}/sandbox}"
KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-spiffe-helper}"
KIND_BIN="${KIND:-kind}"
DOCKER_BIN="${DOCKER:-docker}"

IMAGE_NAME="spiffe-debugger:local"
DOCKERFILE_PATH="${SANDBOX_DIR}/docker/spiffe-debugger.Dockerfile"

"${DOCKER_BIN}" build -f "${DOCKERFILE_PATH}" -t "${IMAGE_NAME}" "${ROOT_DIR}"
"${KIND_BIN}" load docker-image "${IMAGE_NAME}" --name "${KIND_CLUSTER_NAME}"
