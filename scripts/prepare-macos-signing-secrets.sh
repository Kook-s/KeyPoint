#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 /path/to/developer-id-application.p12" >&2
  exit 1
fi

certificate_path="$1"

if [ ! -f "$certificate_path" ]; then
  echo "Certificate file not found: $certificate_path" >&2
  exit 1
fi

openssl base64 -A -in "$certificate_path" -out certificate-base64.txt
LC_ALL=C tr -dc 'A-Za-z0-9' < /dev/urandom | head -c 32 > keychain-password.txt
printf '\n' >> keychain-password.txt

echo "Created certificate-base64.txt for APPLE_CERTIFICATE"
echo "Created keychain-password.txt for KEYCHAIN_PASSWORD"
echo "Use your .p12 export password for APPLE_CERTIFICATE_PASSWORD"
