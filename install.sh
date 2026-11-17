#!/bin/sh
#
# HostBlocker installer — downloads the newest GitHub release and installs it.
#
#   curl -fsSL https://raw.githubusercontent.com/tobrun/host-blocker/main/install.sh | sh
#   curl -fsSL .../install.sh | sh -s -- --uninstall
#
# Uses curl (whose downloads are not quarantined on macOS, so Gatekeeper does
# not block the app) and never pipes anything into an interpreter as root.
#
set -eu

REPO="tobrun/host-blocker"
APP_NAME="HostBlocker"
TAG="${HOSTBLOCKER_TAG:-}" # pin with HOSTBLOCKER_TAG=v0.1.0

fail() { echo "install.sh: $1" >&2; exit 1; }

need() { command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"; }
need curl
need tar

latest_tag() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | head -n1 | cut -d'"' -f4
}

case "$(uname -s)" in
  Darwin) OS=macos ;;
  Linux)  OS=linux ;;
  *) fail "unsupported operating system: $(uname -s)" ;;
esac

[ -n "$TAG" ] || TAG="$(latest_tag)"
[ -n "$TAG" ] || fail "could not determine the latest release of ${REPO}"

RELEASE_URL="https://github.com/${REPO}/releases/download/${TAG}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

asset_name() {
  # $1 = glob suffix; prints the matching asset basename for $TAG.
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/tags/${TAG}" \
    | grep '"name"' | cut -d'"' -f4 | grep -E "$1" | head -n1
}

uninstall() {
  case "$OS" in
    macos)
      [ -d "/Applications/${APP_NAME}.app" ] || { echo "${APP_NAME} is not installed."; exit 0; }
      echo "About to remove /Applications/${APP_NAME}.app"
      echo "Tip: open ${APP_NAME} → Settings → 'Remove all entries' first to"
      echo "clean /etc/hosts before removing the app."
      if [ -w /Applications ]; then rm -rf "/Applications/${APP_NAME}.app"
      else sudo rm -rf "/Applications/${APP_NAME}.app"; fi
      echo "Uninstalled. (State kept in ~/Library/Application Support/app.tobrun.hostblocker)"
      ;;
    linux)
      if command -v dpkg >/dev/null 2>&1 && dpkg -s hostblocker >/dev/null 2>&1; then
        echo "Tip: open ${APP_NAME} → Settings → 'Remove all entries' first to"
        echo "clean /etc/hosts before removing the package."
        sudo apt remove -y hostblocker
        echo "Uninstalled the .deb package."
      elif [ -x "$HOME/.local/bin/${APP_NAME}" ]; then
        rm -f "$HOME/.local/bin/${APP_NAME}" \
              "$HOME/.local/share/applications/hostblocker.desktop" \
              "$HOME/.local/share/icons/hicolor/256x256/apps/hostblocker.png"
        echo "Uninstalled. (State kept in ~/.local/share/app.tobrun.hostblocker)"
      else
        echo "${APP_NAME} is not installed."
      fi
      ;;
  esac
}

install_macos() {
  # tauri-action names the macOS updater tarball without a version when
  # building the universal target (e.g. HostBlocker_universal.app.tar.gz).
  ASSET="$(asset_name "^${APP_NAME}_.*\\.app\\.tar\\.gz$")"
  [ -n "$ASSET" ] || fail "no macOS universal tarball found in release ${TAG}"
  echo "Downloading ${ASSET} ..."
  curl -fL --progress-bar -o "${TMP_DIR}/${ASSET}" "${RELEASE_URL}/${ASSET}"
  tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}"
  [ -d "${TMP_DIR}/${APP_NAME}.app" ] || fail "tarball did not contain ${APP_NAME}.app"
  if [ -w /Applications ]; then
    rm -rf "/Applications/${APP_NAME}.app"
    mv "${TMP_DIR}/${APP_NAME}.app" /Applications/
  else
    sudo rm -rf "/Applications/${APP_NAME}.app"
    sudo mv "${TMP_DIR}/${APP_NAME}.app" /Applications/
  fi
  echo "Installed /Applications/${APP_NAME}.app (${TAG})"
}

install_linux() {
  if command -v dpkg >/dev/null 2>&1; then
    ASSET="$(asset_name "^${APP_NAME}_.*_amd64\\.deb$")"
    [ -n "$ASSET" ] || fail "no .deb asset found in release ${TAG}"
    echo "Downloading ${ASSET} ..."
    curl -fL --progress-bar -o "${TMP_DIR}/${ASSET}" "${RELEASE_URL}/${ASSET}"
    sudo apt install -y "${TMP_DIR}/${ASSET}"
    echo "Installed ${ASSET} (${TAG})"
    return
  fi
  ASSET="$(asset_name "^${APP_NAME}_.*_amd64\\.AppImage$")"
  [ -n "$ASSET" ] || fail "no AppImage asset found in release ${TAG}"
  echo "Downloading ${ASSET} (AppImage fallback, no dpkg found) ..."
  curl -fL --progress-bar -o "${TMP_DIR}/${ASSET}" "${RELEASE_URL}/${ASSET}"
  mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
  install -m 755 "${TMP_DIR}/${ASSET}" "$HOME/.local/bin/${APP_NAME}"
  ICON="$HOME/.local/share/icons/hicolor/256x256/apps/hostblocker.png"
  if APPIMAGE_EXTRACT_AND_RUN=1 "${TMP_DIR}/${ASSET}" --appimage-extract '*.png' >/dev/null 2>&1; then
    EXTRACTED="$(find "${TMP_DIR}" -name '256x256.png' -o -name '*hostblocker*.png' 2>/dev/null | head -n1 || true)"
    if [ -n "$EXTRACTED" ]; then
      mkdir -p "$(dirname "$ICON")"
      install -m 644 "$EXTRACTED" "$ICON"
    fi
  fi
  cat > "$HOME/.local/share/applications/hostblocker.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=${APP_NAME}
Comment=Block websites via /etc/hosts
Exec=$HOME/.local/bin/${APP_NAME}
Icon=${ICON}
Categories=Utility;
EOF
  echo "Installed AppImage to ~/.local/bin/${APP_NAME} (${TAG})"
  echo "Make sure ~/.local/bin is in your PATH."
}

for arg in "$@"; do
  case "$arg" in
    --uninstall) uninstall; exit 0 ;;
    -h|--help)
      sed -n '3,9p' "$0"; exit 0 ;;
    *) fail "unknown argument: $arg (see --help)" ;;
  esac
done

case "$OS" in
  macos) install_macos ;;
  linux) install_linux ;;
esac
