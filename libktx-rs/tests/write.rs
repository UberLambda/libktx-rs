// Copyright (C) 2021 Paolo Jovon <paolo.jovon@gmail.com>
// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "write")]
mod write {
    use libktx_rs::{
        sinks::StreamSink,
        sources::{Ktx1CreateInfo, Ktx2CreateInfo, StreamSource},
        RustKtxStream, Texture, TextureCreateFlags,
    };
    use std::{
        io::{Cursor, Seek, SeekFrom},
        sync::{Arc, Mutex},
    };

    fn write_and_check(texture: &Texture) {
        let cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
        let stream = RustKtxStream::new(Box::new(cursor)).expect("a ktxStream over a io::Cursor");
        let arc_stream = Arc::new(Mutex::new(stream));

        {
            let mut sink = StreamSink::new(arc_stream.clone());
            texture
                .write_to(&mut sink)
                .expect("writing a KTX to io::Cursor");
        }

        // Rewind the stream
        {
            let mut stream_lock = arc_stream.lock().expect("Poisoned stream lock");
            stream_lock
                .inner_mut()
                .seek(SeekFrom::Start(0))
                .expect("rewinding the io::Cursor");
        }

        let source = StreamSource::new(arc_stream.clone(), TextureCreateFlags::LOAD_IMAGE_DATA);
        let written_texture = Texture::new(source);
        written_texture.expect("reading the same KTX back from the cursor");
    }

    #[test]
    fn write_default_ktx1() {
        let texture = Texture::new(Ktx1CreateInfo::default()).expect("a default KTX1 texture");
        write_and_check(&texture);
    }

    #[test]
    fn write_default_ktx2() {
        let texture = Texture::new(Ktx2CreateInfo::default()).expect("a default KTX2 texture");
        write_and_check(&texture);
    }
}
