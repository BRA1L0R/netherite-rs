/// Trait for packets that have
/// an associated ID. Usually
/// implemented alongside [`netherite::Serialize`]
pub trait PacketId {
    /// PacketId
    const ID: i32;
}

impl<T: PacketId> PacketId for &T {
    const ID: i32 = T::ID;
}
