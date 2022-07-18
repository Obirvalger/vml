#!/bin/sh -eux

# Run this tests inside vms created for several distros (from script test-distros.sh)

VML_BIN_DIR="$1"
VM="$2"
USER="$3"
LOG="$HOME/test.log"

export PATH="$VML_BIN_DIR:$PATH"

cd "/home/$USER/src/vml"

echo "$VM": run test install package >> "$LOG"
vml run --nproc "$(nproc)" -m 1G fst --exists-replace --running-restart
vml ssh --check fst -c 'apt-get update'
vml ssh --check fst -c 'apt-get -y install vim-console'
vml stop fst
echo "$VM: done test install package" >> "$LOG"

echo "$VM": run test tap network >> "$LOG"
vml run --nproc "$(nproc)" -m 1G --net-tap tap0 --net-address 172.16.0.2/24 \
    --net-gateway 172.16.0.1 tapnet/fst --exists-replace --running-restart
vml run --nproc "$(nproc)" -m 1G --net-tap tap1 --net-address 172.16.0.3/24 \
    --net-gateway 172.16.0.1 tapnet/snd --exists-replace --running-restart
ping 172.16.0.2 -c 1
ping 172.16.0.3 -c 1
vml rm -f -p tapnet
echo "$VM: done test tap network" >> "$LOG"
