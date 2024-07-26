#!/bin/sh -eux

IMAGES='alt alt-p10 alt-p11 arch fedora ubuntu'

LOG=test.log

:> "$LOG"

for RELEASE in --release ''; do
    cargo build $RELEASE
    test/test-distros.sh "$IMAGES" "$RELEASE"
    test/test-images.sh "$RELEASE"
done
