# Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
# SPDX-License-Identifier: Apache-2.0

function(disable_etc_unpack TARGET)
    get_target_property(KTX_SOURCES ${TARGET} SOURCES)
    list(REMOVE_ITEM KTX_SOURCES
        # This one has the non-free license
        "lib/etcdec.cxx"
        # This one is Apache2, but useless without the other one
        "lib/etcunpack.cxx"
    )
    set_property(TARGET ${TARGET} PROPERTY SOURCES ${KTX_SOURCES})
    target_compile_definitions(${TARGET} PUBLIC -DSUPPORT_SOFTWARE_ETC_UNPACK=0)
endfunction()

option(KTX_BUILD_ETC_UNPACK "Build the non-free ETC unpacker?" OFF)
if(KTX_BUILD_ETC_UNPACK)
    message(STATUS "Building the non-free ETC unpacker")
    # Nothing to do - KTX-Software/CMakeLists.txt builds it by default
else()
    message(STATUS "NOT building the non-free ETC unpacker")
    disable_etc_unpack(ktx) 
    disable_etc_unpack(ktx_read) 
endif()
