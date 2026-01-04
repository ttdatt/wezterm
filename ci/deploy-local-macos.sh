#!/bin/bash
set -euo pipefail

if [[ "${OSTYPE:-}" != darwin* ]] ; then
  echo "ci/deploy-local-macos.sh is intended to run on macOS" >&2
  exit 1
fi

usage() {
  cat <<EOF >&2
Usage:
  ci/deploy-local-macos.sh [target_dir] [codesign_identity]

Examples:
  ci/deploy-local-macos.sh
  ci/deploy-local-macos.sh target
  ci/deploy-local-macos.sh "WezTerm Local"
  ci/deploy-local-macos.sh target "WezTerm Local"

Environment:
  MACOS_CODESIGN_IDENTITY overrides the identity (default: "WezTerm Local")
EOF
}

TARGET_DIR=target
IDENTITY="WezTerm Local"

case $# in
  0)
    ;;
  1)
    if [[ -d "$1" ]] ; then
      TARGET_DIR="$1"
    else
      IDENTITY="$1"
    fi
    ;;
  2)
    TARGET_DIR="$1"
    IDENTITY="$2"
    ;;
  *)
    usage
    exit 2
    ;;
esac

IDENTITY=${MACOS_CODESIGN_IDENTITY:-$IDENTITY}

if [[ -z "${TAG_NAME:-}" ]] ; then
  TAG_NAME=$(git -c "core.abbrev=8" show -s "--format=%cd-%h" "--date=format:%Y%m%d-%H%M%S")
fi

export TAG_NAME
export MACOS_CODESIGN_IDENTITY="$IDENTITY"

./ci/deploy.sh "$TARGET_DIR"

ZIPDIR="WezTerm-macos-$TAG_NAME"
APP="$ZIPDIR/WezTerm.app"

if [[ -d "$APP" ]] ; then
  /usr/bin/codesign --verify --deep --strict "$APP"
  /usr/bin/codesign -dv --verbose=4 "$APP" 2>&1 | grep -E "^(Identifier|TeamIdentifier|Signature|Authority)=" || true
else
  echo "Expected app bundle not found at: $APP" >&2
  exit 1
fi

echo "Wrote: $ZIPDIR.zip"
