#!/bin/bash
# Generate self-signed development certificates for TLS testing
# Requires OpenSSL to be installed

openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout dev-key.pem \
  -out dev-cert.pem \
  -days 365 \
  -subj "/CN=localhost"

echo "Generated dev-cert.pem and dev-key.pem for localhost"
echo "These certificates are for DEVELOPMENT ONLY"
