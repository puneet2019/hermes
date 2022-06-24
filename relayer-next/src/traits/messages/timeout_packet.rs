use async_trait::async_trait;

use crate::traits::relay_context::RelayContext;
use crate::types::aliases::{ChannelId, Height, IbcMessage, PortId, Sequence};

#[async_trait]
pub trait TimeoutUnorderedPacketMessageBuilder: RelayContext {
    async fn build_timeout_unordered_packet_message(
        &self,
        height: Height<Self::DstChain>,
        port_id: PortId<Self::DstChain, Self::SrcChain>,
        channel_id: ChannelId<Self::DstChain, Self::SrcChain>,
        sequence: Sequence<Self::SrcChain, Self::DstChain>,
    ) -> Result<IbcMessage<Self::SrcChain, Self::DstChain>, Self::Error>;
}

#[async_trait]
pub trait TimeoutOrderedPacketMessageBuilder: RelayContext {
    async fn build_timeout_ordered_packet_message(
        &self,
        height: Height<Self::DstChain>,
        port_id: PortId<Self::DstChain, Self::SrcChain>,
        channel_id: ChannelId<Self::DstChain, Self::SrcChain>,
    ) -> Result<IbcMessage<Self::SrcChain, Self::DstChain>, Self::Error>;
}

#[async_trait]
pub trait TimeoutChannelClosedMessageBuilder: RelayContext {
    async fn build_timeout_channel_closed_message(
        &self,
        height: Height<Self::DstChain>,
        port_id: PortId<Self::DstChain, Self::SrcChain>,
        channel_id: ChannelId<Self::DstChain, Self::SrcChain>,
    ) -> Result<IbcMessage<Self::SrcChain, Self::DstChain>, Self::Error>;
}