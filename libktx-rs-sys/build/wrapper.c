#include "wrapper.h"

KTX_API KTX_error_code KTX_APIENTRY ktxTexture_CreateFromStream(ktxStream *pStream, ktxTextureCreateFlags createFlags, ktxTexture **newTex)
{
    return ktxTexture_createFromStream(pStream, createFlags, newTex);
}