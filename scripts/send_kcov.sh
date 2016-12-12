#!/bin/bash

PKGID="$(cargo pkgid)"
[ -z "$PKGID" ] && exit 1
ORIGIN="${PKGID%#*}"
ORIGIN="${ORIGIN:7}"
PKGNAMEVER="${PKGID#*#}"
PKGNAME="$(echo $ORIGIN | sed 's!.*\/!!')"
shift
cargo test --no-run || exit $?

for i in $ORIGIN/target/debug/${PKGNAME}*
do
    echo "send_kcov: $i"
    OUTDIR="target/cov/$(basename $i)"
    mkdir -p "${OUTDIR}"
    kcov/build/src/kcov \
        --exclude-pattern=/.cargo,/usr/lib \
        --verify \
        --coveralls-id=${TRAVIS_JOB_ID} \
        "${OUTDIR}" \
        "$i"
done
