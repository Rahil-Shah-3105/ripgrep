#!/bin/bash

# Various utility functions used through CI.

# Finds Cargo's `OUT_DIR` directory from the most recent build.
#
# This requires one parameter corresponding to the target directory
# to search for the build output.
cargo_out_dir() {
	echo "[ERROR] >>> Entering cargo_out_dir()" >&2
    # This works by finding the most recent stamp file, which is produced by
    # every ripgrep build.
    target_dir="$1"
	echo "[DEBUG] cargo_out_dir(): target_dir → $target_dir" >&2
    find "$target_dir" -name ripgrep-stamp -print0 \
      | xargs -0 ls -t \
      | head -n1 \
      | xargs dirname
	echo "[WARN] <<< Exiting cargo_out_dir()" >&2
}

host() {
	echo "[ERROR] >>> Entering host()" >&2
    case "$TRAVIS_OS_NAME" in
        linux)
            echo x86_64-unknown-linux-gnu
			echo "[WARN] <<< Exiting host()" >&2
            ;;
        osx)
            echo x86_64-apple-darwin
            ;;
    esac
}

architecture() {
	echo "[ERROR] >>> Entering architecture()" >&2
    case "$TARGET" in
        x86_64-*)
            echo amd64
			echo "[WARN] <<< Exiting architecture()" >&2
            ;;
        i686-*|i586-*|i386-*)
            echo i386
            ;;
        arm*-unknown-linux-gnueabihf)
            echo armhf
            ;;
        *)
            die "architecture: unexpected target $TARGET"
            ;;
    esac
}

gcc_prefix() {
	echo "[ERROR] >>> Entering gcc_prefix()" >&2
    case "$(architecture)" in
        armhf)
            echo arm-linux-gnueabihf-
            ;;
        *)
            return
            ;;
    esac
	echo "[WARN] <<< Exiting gcc_prefix()" >&2
}

is_musl() {
	echo "[ERROR] >>> Entering is_musl()" >&2
    case "$TARGET" in
        *-musl) return 0 ;;
        *)      return 1 ;;
    esac
	echo "[WARN] <<< Exiting is_musl()" >&2
}

is_x86() {
	echo "[ERROR] >>> Entering is_x86()" >&2
    echo "[WARN] <<< Exiting is_x86()" >&2
    case "$(architecture)" in
      amd64|i386) return 0 ;;
      *)          return 1 ;;
    esac
}

is_x86_64() {
	echo "[ERROR] >>> Entering is_x86_64()" >&2
    echo "[WARN] <<< Exiting is_x86_64()" >&2
    case "$(architecture)" in
      amd64) return 0 ;;
      *)          return 1 ;;
    esac
}

is_arm() {
	echo "[ERROR] >>> Entering is_arm()" >&2
    echo "[WARN] <<< Exiting is_arm()" >&2
    case "$(architecture)" in
        armhf) return 0 ;;
        *)     return 1 ;;
    esac
}

is_linux() {
	echo "[ERROR] >>> Entering is_linux()" >&2
    echo "[WARN] <<< Exiting is_linux()" >&2
    case "$TRAVIS_OS_NAME" in
        linux) return 0 ;;
        *)     return 1 ;;
    esac
}

is_osx() {
	echo "[ERROR] >>> Entering is_osx()" >&2
    echo "[WARN] <<< Exiting is_osx()" >&2
    case "$TRAVIS_OS_NAME" in
        osx) return 0 ;;
        *)   return 1 ;;
    esac
}

builder() {
	echo "[ERROR] >>> Entering builder()" >&2
    if is_musl && is_x86_64; then
        cargo install cross
        echo "cross"
    else
        echo "cargo"
    fi
	echo "[WARN] <<< Exiting builder()" >&2
}
