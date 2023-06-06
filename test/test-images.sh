#!/bin/sh -eux

RELEASE="${1-}"
VML_SRC="${2-"$(pwd)"}"
USER=builder
LOG=test.log

if [ -n "$RELEASE" ]; then
    VML_BIN_DIR="$VML_SRC/target/release"
    VML_BIN_DIR_INNER="/home/$USER/src/vml/target/release"
else
    VML_BIN_DIR="$VML_SRC/target/debug"
    VML_BIN_DIR_INNER="/home/$USER/src/vml/target/debug"
fi
export PATH="$VML_BIN_DIR:$PATH"
cd "$VML_SRC"

VM="TEST_VML/TEST_IMAGES"
vml run --no-ssh --nproc "$(nproc)" -m 3G -n "$VM" --ssh-user "$USER" \
    --exists-replace --running-restart
echo run vm "$VM" >> "$LOG"

ansible-playbook test/playbook.yaml \
    -i files/scripts/inventory.py \
    -e ansible_user=root \
    -e vml_src="$VML_SRC" \
    -e parent="$VM" \
    -e user="$USER"
echo provision vm "$VM" >> "$LOG"

vml ssh -n "$VM" --check -c 'mkdir -p src bin'
vml rsync-to -avP -n "$VM" -s ./ -d src/vml/ --rsync-options --exclude target
vml ssh -n "$VM" --check -c "sh -c 'cd src/vml; cargo build $RELEASE'"
vml ssh -n "$VM" -c "ln -s $VML_BIN_DIR_INNER ~/bin"
echo build vml on vm "$VM" >> "$LOG"

vml ssh -n "$VM" --check \
    -c "/home/$USER/src/vml/test/test-images-inner.sh $VML_BIN_DIR_INNER $VM"
vml ssh -n "$VM" -c "cat $LOG" >> "$LOG"

vml rm -f "$VM"
