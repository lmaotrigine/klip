#!/bin/sh

set -eux

TMPDIR=${TMPDIR:-/tmp}
TARGET=${TARGET:-x86_64-unknown-linux-gnu}
#cargo +nightly -Zbuild-std=std,panic_abort build --target "${TARGET}" --release --locked -p klip
__cmdline="target/${TARGET}/release/klip"
KLIP_S="${__cmdline} -c ${TMPDIR}/klip-test-server.toml serve"
KLIP_C="${__cmdline} -c ${TMPDIR}/klip-test-client.toml"

cat > "${TMPDIR}/klip-test-server.toml" <<EOF
listen     = "127.0.0.1:8076"
psk        = "0768e822c681c887e1c80ad57412aca814738bdd4380f00445d159d80d4c28c4"
sign_pk    = "6bdf6e6e0cfdcf6fd4991a9b27114a29fba538f91f84297892612a1b1767f58c"
EOF

cat > "${TMPDIR}/klip-test-client.toml" <<EOF
connect    = "127.0.0.1:8076"
psk        = "0768e822c681c887e1c80ad57412aca814738bdd4380f00445d159d80d4c28c4"
sign_pk    = "6bdf6e6e0cfdcf6fd4991a9b27114a29fba538f91f84297892612a1b1767f58c"
sign_sk    = "3128a243ebf8f78dbf551741775f9da36d1ac0ffa11dc7776d038310ea762f2f"
encrypt_sk = "28956ee776dc1f171e652472c6a27ce27166bf58effa10b574dbe159128d8898"
EOF

${KLIP_S} &
pid=$!
sleep 2
dd if=/dev/urandom of=/tmp/kl bs=1000 count=1
${KLIP_C} copy < /tmp/kl
${KLIP_C} paste > /tmp/kl2
cmp /tmp/kl /tmp/kl2
${KLIP_C} paste | ${KLIP_C} copy
${KLIP_C} move > /tmp/kl2
cmp /tmp/kl /tmp/kl2
${KLIP_C} paste && exit 1
kill $pid
wait $pid || true

echo
echo 'Success!'
echo
