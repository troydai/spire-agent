#!/bin/bash
# Deploys the SPIRE CSI driver to the Kubernetes cluster.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[deploy-csi] Deploying SPIRE CSI driver..."
kubectl apply -f "${SCRIPT_DIR}/../../deploy/spire/csi/"
echo -e "✓ SPIRE CSI driver deployed"
