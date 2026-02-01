#!/bin/bash
# Removes the SPIRE CSI driver from the cluster.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

ROOT_DIR="${ROOT_DIR:-$(cd "${SCRIPT_DIR}/../.." && pwd)}"
KUBECONFIG_PATH="${KUBECONFIG_PATH:-${ROOT_DIR}/artifacts/kubeconfig}"

export KUBECONFIG="${KUBECONFIG_PATH}"

echo -e "[undeploy-csi] Undeploying SPIRE CSI driver..."
kubectl delete -f "${SCRIPT_DIR}/../../deploy/spire/csi/" --ignore-not-found
echo -e "✓ SPIRE CSI driver undeployed"
