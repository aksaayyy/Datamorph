#!/bin/bash
# Create GitHub release and upload assets
# Usage: ./scripts/create-release.sh v0.1.0 path/to/assets/*

set -e

if [ $# -lt 2 ]; then
  echo "Usage: $0 <tag> <asset1> [<asset2> ...]"
  echo "Example: $0 v0.1.0 release/datamorph-linux-amd64 release/datamorph-macos-amd64"
  exit 1
fi

TAG="$1"
shift
ASSETS="$@"

# Check if GITHUB_TOKEN is set
if [ -z "$GITHUB_TOKEN" ]; then
  echo "❌ Error: GITHUB_TOKEN environment variable not set"
  echo "Get your token from: https://github.com/settings/tokens"
  echo "Then run: export GITHUB_TOKEN=your_token_here"
  exit 1
fi

# Create release (if tag doesn't exist, create annotated tag)
echo "🎉 Creating release for tag: $TAG"
RELEASE_JSON=$(curl -s -X POST \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  https://api.github.com/repos/aksaayyy/Datamorph/releases \
  -d "$(jq -n --arg tag "$TAG" --arg name "$TAG" --arg body "Datamorph $TAG release" '{tag_name:$tag,name:$name,body:$body,draft:false,prerelease:false}')")

RELEASE_ID=$(echo "$RELEASE_JSON" | jq -r '.id')
if [ "$RELEASE_ID" == "null" ] || [ -z "$RELEASE_ID" ]; then
  echo "❌ Failed to create release:"
  echo "$RELEASE_JSON" | jq -r '.message'
  exit 1
fi

echo "✅ Release created (ID: $RELEASE_ID)"

# Upload assets
for ASSET in $ASSETS; do
  if [ ! -f "$ASSET" ]; then
    echo "⚠️  Skipping missing file: $ASSET"
    continue
  fi

  echo "⬆️  Uploading: $ASSET"
  FILE_NAME=$(basename "$ASSET")
  RESPONSE=$(curl -s -X POST \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "Accept: application/vnd.github.v3+json" \
    -H "Content-Type: application/octet-stream" \
    --data-binary @"$ASSET" \
    "https://api.github.com/repos/aksaayyy/Datamorph/releases/$RELEASE_ID/assets?name=$FILE_NAME")

  STATUS=$(echo "$RESPONSE" | jq -r '.state // .message')
  if [ "$STATUS" == "uploaded" ] || echo "$RESPONSE" | grep -q '"id"'; then
    echo "   ✅ $FILE_NAME uploaded"
  else
    echo "   ❌ Failed: $RESPONSE"
  fi
done

echo "🎉 Release $TAG is live! https://github.com/aksaayyy/Datamorph/releases/tag/$TAG"
