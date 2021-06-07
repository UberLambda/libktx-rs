SELF_DIR=$(dirname $0)
bindgen \
    --opaque-type 'FILE' \
    --allowlist-function 'ktx.*' --allowlist-type '[Kk][Tt][Xx].*' --allowlist-var '[Kk][Tt][Xx].*' \
    --output "${SELF_DIR}/../src/ffi.rs" \
    "${SELF_DIR}/KTX-Software/include/ktx.h" \
    -- -fparse-all-comments -U_MSC_VER
