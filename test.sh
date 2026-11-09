#!/bin/sh

set -eux

TMPDIR=${TMPDIR:-/tmp}
TARGET=${TARGET:+"/$TARGET"}
__cmdline="${KLIP:-target${TARGET}/debug/klip}"
KLIP_S="${__cmdline} -c ${TMPDIR}/klip-test-server.toml serve"
KLIP_C="${__cmdline} -c ${TMPDIR}/klip-test-client.toml"

set +x
keys=$($__cmdline genkeys)
echo "$keys" | tail -n +8 | head -n 5 > "${TMPDIR}/klip-test-client.toml"
echo "$keys" | tail -n +16 | head -n 3 > "${TMPDIR}/klip-test-server.toml"
set -x

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
