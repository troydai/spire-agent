#!/usr/bin/env bash
# Generates CA and SPIRE server certificates for the local development environment.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

info() {
  echo -e "[certs] $*"
}

error() {
  echo -e "[certs] $*" >&2
}

# Default values
CERT_DIR="${CERT_DIR:-./artifacts/certs}"
FORCE="${FORCE:-0}"

# Certificate configuration
CA_KEY="${CERT_DIR}/ca-key.pem"
CA_CERT="${CERT_DIR}/ca-cert.pem"
SERVER_KEY="${CERT_DIR}/spire-server-key.pem"
SERVER_CERT="${CERT_DIR}/spire-server-cert.pem"
SERVER_CSR="${CERT_DIR}/spire-server.csr"
BOOTSTRAP_BUNDLE="${CERT_DIR}/bootstrap-bundle.pem"

# Create certs directory if it doesn't exist
mkdir -p "${CERT_DIR}"

# Check if openssl is available
if ! command -v openssl >/dev/null 2>&1; then
  error "openssl is required but not found. Install it and re-run."
  exit 1
fi

# Function to check if a file exists and is non-empty
file_exists() {
  [[ -f "$1" ]] && [[ -s "$1" ]]
}

# Function to generate CA certificate and key
generate_ca() {
  if [[ "${FORCE}" -eq 0 ]] && file_exists "${CA_KEY}" && file_exists "${CA_CERT}"; then
    echo -e "[certs] CA certificate and key already exist, skipping generation"
    return 0
  fi

  echo -e "[certs] Generating CA private key (RSA 4096) in PKCS#8 format..."
  openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:4096 -out "${CA_KEY}"

  echo -e "[certs] Generating CA certificate..."
  openssl req -new -x509 -days 3650 -key "${CA_KEY}" -out "${CA_CERT}" \
    -subj "/CN=spiffe-helper-sandbox-ca" \
    -addext "basicConstraints=critical,CA:TRUE" \
    -addext "keyUsage=critical,keyCertSign,cRLSign"

  echo -e "✓ CA certificate and key generated successfully"
}

# Function to generate SPIRE server certificate and key
generate_spire_server() {
  if [[ "${FORCE}" -eq 0 ]] && file_exists "${SERVER_KEY}" && file_exists "${SERVER_CERT}"; then
    echo -e "[certs] SPIRE server certificate and key already exist, skipping generation"
    return 0
  fi

  echo -e "[certs] Generating SPIRE server private key (RSA 2048) in PKCS#8 format..."
  openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out "${SERVER_KEY}"

  echo -e "[certs] Generating SPIRE server certificate signing request..."
  openssl req -new -key "${SERVER_KEY}" -out "${SERVER_CSR}" \
    -subj "/CN=spiffe-helper-sandbox-spire-server"

  echo -e "[certs] Generating SPIRE server certificate (signed by CA)..."
  openssl x509 -req -in "${SERVER_CSR}" -CA "${CA_CERT}" -CAkey "${CA_KEY}" \
    -CAcreateserial -out "${SERVER_CERT}" -days 365 \
    -extensions v3_server -extfile <(
      echo "[v3_server]"
      echo "basicConstraints=CA:FALSE"
      echo "keyUsage=digitalSignature"
      echo "extendedKeyUsage=serverAuth"
      echo "subjectAltName=DNS:spire-server,DNS:spire-server.default.svc.cluster.local,DNS:spire-server.spire-server.svc.cluster.local"
    )

  echo -e "✓ SPIRE server certificate and key generated successfully"
}

# Function to generate bootstrap bundle
generate_bootstrap_bundle() {
  if [[ "${FORCE}" -eq 0 ]] && file_exists "${BOOTSTRAP_BUNDLE}"; then
    echo -e "[certs] Bootstrap bundle already exists, skipping generation"
    return 0
  fi

  if ! file_exists "${CA_CERT}"; then
    error "CA certificate not found. Cannot generate bootstrap bundle."
    exit 1
  fi

  echo -e "[certs] Generating bootstrap bundle..."
  cp "${CA_CERT}" "${BOOTSTRAP_BUNDLE}"

  echo -e "✓ Bootstrap bundle generated successfully"
}

main() {
  echo -e "[certs] Starting certificate generation..."
  echo -e "[certs] Output directory: ${CERT_DIR}"

  generate_ca
  generate_spire_server
  generate_bootstrap_bundle

  echo ""
  echo -e "[certs] Certificate generation complete!"
  echo ""
  echo -e "[certs] Generated files:"
  echo -e " ✓ CA: ${CA_KEY}, ${CA_CERT}"
  echo -e " ✓ SPIRE Server: ${SERVER_KEY}, ${SERVER_CERT}, ${SERVER_CSR}"
  echo -e " ✓ Bootstrap Bundle: ${BOOTSTRAP_BUNDLE}"
}

main "$@"
