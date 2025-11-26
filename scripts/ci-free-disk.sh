#!/usr/bin/env bash
# Free disk space on CI runners by removing unnecessary files

set -euo pipefail

echo "Disk space before cleanup:"
df -h

# Remove unnecessary tools and packages
sudo rm -rf /usr/share/dotnet
sudo rm -rf /opt/ghc
sudo rm -rf /usr/local/share/boost
sudo rm -rf "$AGENT_TOOLSDIRECTORY"

# Clean up Docker resources
docker system prune -af --volumes

echo "Disk space after cleanup:"
df -h
