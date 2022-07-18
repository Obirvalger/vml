#!/bin/sh -eux

# Run this tests inside vm (from script test-images.sh)

VML_BIN_DIR="$1"
VM="$2"
LOG="$HOME/test.log"
BAD_IMAGES="$HOME/bad-images"

:> "$BAD_IMAGES"

export PATH="$VML_BIN_DIR:$PATH"

for IMAGE in $(vml image available | awk '{print $1}' | grep -v -- -gui); do
    echo "$VM: run image $IMAGE" >> "$LOG"
    vml run --nproc "$(nproc)" -N "$IMAGE" || echo "Failed run $IMAGE" >> "$BAD_IMAGES"
    vml ssh -n "$IMAGE" --check -c 'true' || echo "Failed ssh $IMAGE" >> "$BAD_IMAGES"
    vml rm -f "$IMAGE"
    vml image rm "$IMAGE"
    echo "$VM: done image $IMAGE" >> "$LOG"
done

cat "$BAD_IMAGES" >> "$LOG"
