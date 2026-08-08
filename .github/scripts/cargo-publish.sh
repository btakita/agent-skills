#!/usr/bin/env bash
# Publish to crates.io, tolerating ONLY the genuinely idempotent case: this
# version is already on crates.io (e.g. a re-run of an existing tag).
#
# `cargo publish --allow-dirty || true` used to swallow every failure here, so a
# broken publish still reported a green job: v0.1.3 failed with `403
# authentication failed` (expired CARGO_REGISTRY_TOKEN) while the run showed
# `publish: success`, and the crate had to be published by hand. Everything
# except the duplicate-version case must fail the job.
set -uo pipefail

log="${PUBLISH_LOG:-publish.log}"
cargo_bin="${CARGO:-cargo}"

if "$cargo_bin" publish "$@" 2>&1 | tee "$log"; then
  exit 0
fi

# Two different wordings mean "this version is already published":
#   crates.io server : crate version `0.1.3` is already uploaded
#   cargo client     : crate skill-harness@0.1.3 already exists on registry `crates-io`
# The client pre-check (`verify_unpublished`) usually fires first, but a stale
# or lagging registry index falls through to the server message, so match both.
if grep -Eq "is already uploaded|already exists on " "$log"; then
  echo "::notice::version already published to crates.io; treating as success"
  exit 0
fi

echo "::error::cargo publish failed"
exit 1
