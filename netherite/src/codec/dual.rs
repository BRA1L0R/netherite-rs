use super::{CompressedCodec, UncompressedCodec};

enum SwitchCodec {
    Uncompressed(UncompressedCodec),
    Compressed(CompressedCodec),
}

impl From<UncompressedCodec> for SwitchCodec {
    fn from(value: UncompressedCodec) -> Self {
        Self::Uncompressed(value)
    }
}

impl From<CompressedCodec> for SwitchCodec {
    fn from(value: CompressedCodec) -> Self {
        Self::Compressed(value)
    }
}

impl SwitchCodec {
    pub fn max_size(self, max_size: usize) -> Self {
        match self {
            Self::Uncompressed(codec) => Self::Uncompressed(codec.max_size(max_size)),
            Self::Compressed(codec) => Self::Compressed(codec.max_size(max_size)),
        }
    }

    pub fn get_max_size(&self) -> usize {
        let &(SwitchCodec::Uncompressed(UncompressedCodec { max_size, .. })
        | SwitchCodec::Compressed(CompressedCodec { max_size, .. })) = self;

        max_size
    }

    // sets whether the switch codec should process packet
    // - using the uncompressed codec (`threshold = None`)
    // - or through the compressed codec (`threshold = Some(..)`)
    // pub fn set_treshold(&mut self, threshold: Option<usize>) {
    //     match threshold {
    //         Some(threshold) => self.set_compressed(threshold),
    //         None => self.set_uncompressed(),
    //     }
    // }

    // pub fn set_compressed(&mut self, threshold: usize) {
    //     match self {
    //         SwitchCodec::Compressed(compressed) => compressed.set_compression(threshold),
    //         SwitchCodec::Uncompressed(MinecraftCodec { max_size, .. }) => {
    //             let minecraft_compressed = MinecraftCompressed::default()
    //                 .max_size(*max_size)
    //                 .compression(threshold);

    //             *self = SwitchCodec::Compressed(minecraft_compressed);
    //         }
    //     };
    // }

    // pub fn set_uncompressed(&mut self) {
    //     let SwitchCodec::Compressed(MinecraftCompressed { max_size, .. }) = self else {
    //         return;
    //     };

    //     // *self = SwitchCodec::Uncompressed(MinecraftCodec { max_size })
    //     todo!()
    // }
}
