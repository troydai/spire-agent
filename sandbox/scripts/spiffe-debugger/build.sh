#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
# If invoked from sandbox with ROOT_DIR pointing at the sandbox dir, correct it.
if [ "$(basename "${ROOT_DIR}")" = "sandbox" ]; then
	ROOT_DIR="$(cd "${ROOT_DIR}/.." && pwd)"
	SANDBOX_DIR="${SANDBOX_DIR:-$(cd "${ROOT_DIR}/sandbox" && pwd)}"
else
	SANDBOX_DIR="${SANDBOX_DIR:-${ROOT_DIR}/sandbox}"
fi
KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-spiffe-helper}"
KIND_BIN="${KIND:-kind}"
DOCKER_BIN="${DOCKER:-docker}"

IMAGE_NAME="spiffe-debugger:local"
DOCKERFILE_PATH="${SANDBOX_DIR}/docker/spiffe-debugger.Dockerfile"

"${DOCKER_BIN}" build -f "${DOCKERFILE_PATH}" -t "${IMAGE_NAME}" "${ROOT_DIR}"
"${KIND_BIN}" load docker-image "${IMAGE_NAME}" --name "${KIND_CLUSTER_NAME}"
