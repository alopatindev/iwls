#!/bin/bash

PKGID="$(cargo pkgid)"
[ -z "$PKGID" ] && exit 1
ORIGIN="${PKGID%#*}"
ORIGIN="${ORIGIN:7}"
PKGNAMEVER="${PKGID#*#}"
PKGNAME="$(echo $ORIGIN | sed 's!.*\/!!')"
shift
cargo test --no-run || exit $?
EXE=($ORIGIN/target/debug/$PKGNAME-*)
if [ ${#EXE[@]} -ne 1 ]; then
    echo 'Non-unique test file, retrying...' >2
    rm -f ${EXE[@]}
    cargo test --no-run || exit $?
fi

kcov/build/src/kcov \
    --exclude-pattern=/.cargo,/usr/lib \
    --verify \
    --coveralls-id=$TRAVIS_JOB_ID \
    $ORIGIN/target/cov \
    $ORIGIN/target/debug/$PKGNAME-* \
    "$@"
