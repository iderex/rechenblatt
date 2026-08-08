#!/usr/bin/env bash
# Run the default suite in an environment that has none of the things the suite
# is not allowed to need, and prove the environment really has none of them.
#
# Usage:
#   .github/scripts/run-sealed.sh suite    the default suite, which must pass
#   .github/scripts/run-sealed.sh probes   the probes, which must all fail
#
# What is taken away, and how:
#
#   no network            --network none, so a socket has no route at all
#   no elevated rights    --user with the calling user's ids, and the script
#                         refuses to continue if it lands as root anyway
#   no writable host      --read-only, so only the workspace and one tmpfs can
#                         be written
#   no display server     nothing sets DISPLAY or WAYLAND_DISPLAY, and there is
#                         no socket for one to point at
#   no host fonts         the image carries no font directory
#
# The image is pinned by digest rather than by tag. A tag moves; the bytes this
# gate ran against must not. The digest was read from the registry rather than
# copied from a local pull:
#
#   token=$(curl -s 'https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/rust:pull' \
#     | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')
#   curl -sI -H "Authorization: Bearer $token" \
#     -H 'Accept: application/vnd.oci.image.index.v1+json' \
#     https://registry-1.docker.io/v2/library/rust/manifests/1.97.1-slim \
#     | sed -n 's/^[Dd]ocker-[Cc]ontent-[Dd]igest: //p'
#
# Exit 0 means the mode's expectation held. Exit 1 means it did not. Exit 2 means
# the run could not happen, which is a failure and never a pass.

set -euo pipefail

mode=${1:-}
case "$mode" in
  suite | probes) ;;
  *)
    echo "usage: $0 suite|probes" >&2
    exit 2
    ;;
esac

# rust:1.97.1-slim, which is the channel rust-toolchain.toml pins.
IMAGE="rust@sha256:3b2879047d42784ca9403ad20c51ed3df361a50f1df96f5777d39b4e33aa65cd"

if ! command -v docker >/dev/null 2>&1; then
  echo "run-sealed: docker is not installed, so the sealed environment cannot be built" >&2
  echo "run-sealed: failing closed rather than reporting a run that did not happen" >&2
  exit 2
fi

if ! docker version --format '{{.Server.Version}}' >/dev/null 2>&1; then
  echo "run-sealed: the docker daemon cannot be reached, so the sealed environment cannot be built" >&2
  echo "run-sealed: failing closed rather than reporting a run that did not happen" >&2
  exit 2
fi

root=$(git rev-parse --show-toplevel)
cd "$root"

channel=$(sed -n 's/^channel = "\(.*\)"$/\1/p' rust-toolchain.toml)
if [ -z "$channel" ]; then
  echo "run-sealed: rust-toolchain.toml declares no channel this script can read" >&2
  exit 2
fi

sealed() {
  docker run --rm \
    --network none \
    --read-only \
    --user "$(id -u):$(id -g)" \
    --tmpfs /tmp:rw,exec,size=1g \
    --env HOME=/tmp/home \
    --env TMPDIR=/tmp \
    --env RUSTUP_TOOLCHAIN="$channel" \
    --env CARGO_HOME=/w/target/sealed/cargo-home \
    --env CARGO_TARGET_DIR=/w/target/sealed \
    --env CARGO_TERM_COLOR=never \
    --volume "$root:/w" \
    --workdir /w \
    "$IMAGE" \
    bash -c "$1"
}

# The preamble every mode runs first. It refuses to go on if the container turned
# out to be privileged after all, because a suite that passed as root proves
# nothing about a suite run by a person.
preamble='
set -euo pipefail
mkdir -p "$HOME"
if [ "$(id -u)" -eq 0 ]; then
  echo "run-sealed: this landed as root, which is not the environment being tested"
  exit 1
fi
echo "uid=$(id -u) gid=$(id -g)"
echo "DISPLAY=${DISPLAY:-<unset>} WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-<unset>}"
rustc --version
'

# What the working tree looks like to git, which is how a file a test left behind
# is noticed. The build directory is ignored, so the derived output the run is
# supposed to produce does not read as a stray write.
tree_state() {
  git status --porcelain --untracked-files=all
}

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
  echo "run-sealed: fetching the pinned image (this is the one step that uses the network)"
  if ! docker pull --quiet "$IMAGE" >/dev/null; then
    echo "run-sealed: the pinned image could not be fetched, so nothing was run" >&2
    exit 2
  fi
fi

case "$mode" in
  suite)
    echo "run-sealed: the default suite, which must pass"

    before=$(tree_state)
    sealed "$preamble
      cargo test --locked --offline --workspace"
    after=$(tree_state)

    # A test writing outside the path it made for itself is the failure this
    # half is about. The container answers for everything outside the workspace,
    # which is read-only there and which probe/write-outside is the proof of.
    # Inside the workspace the container answers for nothing, because the
    # workspace is the one place it must be able to write, so the comparison
    # below is what covers it.
    if [ "$before" != "$after" ]; then
      echo "run-sealed: the suite left the working tree changed, so a test wrote outside the path it made"
      diff <(printf '%s\n' "$before") <(printf '%s\n' "$after") | sed 's/^/  /' || true
      exit 1
    fi

    echo "run-sealed: the default suite is green with no display, no network, no host fonts and no elevated rights"
    echo "run-sealed: and it left the working tree as it found it"
    ;;

  probes)
    echo "run-sealed: the probes, every one of which must fail here"
    log=$(mktemp)
    # shellcheck disable=SC2064
    trap "rm -f '$log'" EXIT

    if sealed "$preamble
      cargo test --locked --offline --workspace -- --ignored" > "$log" 2>&1; then
      echo "run-sealed: the probes PASSED inside the sealed environment, so it is not sealed"
      sed 's/^/  /' "$log"
      exit 1
    fi

    bad=0
    for marker in probe/display probe/host-fonts probe/network probe/write-outside; do
      if ! line=$(grep -m1 -F "$marker" "$log"); then
        echo "run-sealed: no probe reported $marker, so that half of the environment is unproven"
        bad=1
        continue
      fi

      # A probe that timed out and a probe that hung read the same on a log, and
      # neither says the thing was absent. The refusal has to carry a cause.
      if printf '%s' "$line" | grep -qiE 'timed out|timeout'; then
        echo "run-sealed: $marker failed on a timeout, which names no cause:"
        echo "  ${line#*: }"
        bad=1
        continue
      fi

      echo "refused: ${line#*"$marker"}"
    done

    if [ "$bad" -ne 0 ]; then
      sed 's/^/  /' "$log"
      exit 1
    fi
    echo "run-sealed: every probe failed here, and each one named a cause rather than a timeout"
    ;;
esac
