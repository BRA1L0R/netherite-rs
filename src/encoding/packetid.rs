/// Trait for packets that have
/// an associated ID. Usually
/// implemented alongside [`netherite::Serialize`]
pub trait PacketId {
    /// PacketId
    const ID: i32;
}
