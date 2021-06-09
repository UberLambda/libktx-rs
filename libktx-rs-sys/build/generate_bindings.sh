# Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
# SPDX-License-Identifier: Apache-2.0

SELF_DIR=$(dirname $0)

bindgen \
    --opaque-type 'FILE' \
    --allowlist-function 'ktx.*' --allowlist-type '[Kk][Tt][Xx].*' --allowlist-var '[Kk][Tt][Xx].*' \
    --output "${SELF_DIR}/../src/ffi.rs" \
    "${SELF_DIR}/wrapper.h" \
    -- \
    -fparse-all-comments -U_MSC_VER \
    -I "${SELF_DIR}/KTX-Software/include" -I "${SELF_DIR}/KTX-Software/lib" \
