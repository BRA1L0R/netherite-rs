// mod uncompressed {
//     use futures::SinkExt;
//     use netherite::{codec::MinecraftCodec, packet::RawPacket};
//     use tokio_util::codec::{Framed, FramedWrite};

//     #[tokio::test]
//     async fn serialize() {
//         let target = vec![];
//         let mut codec = FramedWrite::new(target, MinecraftCodec::compressed(9));

//         let packet = RawPacket {
//             packet_id: 0xffff,
//             data: [0x00, 0x01, 0x02].as_slice().into(),
//         };

//         codec.feed(packet).await.unwrap();
//         codec.flush().await.unwrap();
//         let inner = codec.into_inner();

//         panic!("{inner:?}");
//     }
// }
