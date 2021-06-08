#pragma once

#include <ktx.h>
#include <stream.h>
#include <texture.h>

// Since the function in texture.h is private, re-export it here
KTX_error_code ktxTexture_createFromStream(ktxStream *pStream, ktxTextureCreateFlags createFlags, ktxTexture **newTex);

KTX_API KTX_error_code KTX_APIENTRY ktxTexture_CreateFromStream(ktxStream *pStream, ktxTextureCreateFlags createFlags, ktxTexture **newTex);