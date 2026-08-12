#!/usr/bin/env bash
# je lance un serveur local pour le guide Toolé (Docsify exige http, pas file://)
# usage : ./serve-guide.sh [port]
set -euo pipefail
PORT="${1:-8000}"
cd "$(dirname "$0")"
echo "Toolé site + guide : http://localhost:${PORT}/"
echo "  site vitrine : http://localhost:${PORT}/"
echo "  guide        : http://localhost:${PORT}/guide/"
echo "Ctrl-C pour arrêter."
python3 -m http.server "${PORT}"