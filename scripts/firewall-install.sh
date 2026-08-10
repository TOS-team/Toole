#!/usr/bin/env bash
# Active le pare-feu pour Toolé (découverte UDP 58199 + transfert QUIC UDP 58200).
# Usage : sudo ./scripts/firewall-install.sh
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Erreur : relance avec sudo (root requis)." >&2
  exit 1
fi

if ! command -v firewall-cmd >/dev/null 2>&1; then
  echo "firewalld n'est pas installé sur cette machine ; rien à faire ici." >&2
  exit 0
fi

REPO_SERVICE="$(cd "$(dirname "$0")/.." && pwd)/assets/firewalld/toole.xml"
SERVICE_FILE="/etc/firewalld/services/toole.xml"

install -m 644 "$REPO_SERVICE" "$SERVICE_FILE"
firewall-cmd --reload

# Applique le service en permanent sur toutes les zones actives
for zone in $(firewall-cmd --get-active-zones | grep -v '^[[:space:]]'); do
  firewall-cmd --zone="$zone" --permanent --add-service=toole >/dev/null
done

firewall-cmd --reload

echo "Pare-feu Toolé activé :"
firewall-cmd --list-services | tr ' ' '\n' | grep '^toole$' || echo "  (service non visible, vérifie manuellement)"