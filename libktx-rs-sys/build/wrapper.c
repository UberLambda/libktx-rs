// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#include "wrapper.h"

KTX_API KTX_error_code KTX_APIENTRY ktxTexture_CreateFromStream(ktxStream *pStream, ktxTextureCreateFlags createFlags, ktxTexture **newTex)
{
    return ktxTexture_createFromStream(pStream, createFlags, newTex);
}