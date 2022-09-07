use crate::core::traits::sync::Async;
use crate::one_for_all::traits::components::OfaComponents;

pub trait OfaChainTypes: Async {
    type Error: Async;

    type Event: Async;

    type Height: Async;

    type Timestamp: Ord + Async;

    type Duration: Ord + Async;

    type Message: Async;

    type MessageType: Eq + Async;

    type Signer: Async;

    type ClientId: Async;

    type ConnectionId: Async;

    type ChannelId: Async;

    type Port: Async;

    type MerkleProof: Async;

    type ClientType: Eq + Async;

    type AnyClientState: Async;

    type AnyConsensusState: Async;

    type AnyClientHeader: Async;

    type AnyMisbehavior: Async;
}

pub trait OfaChain: OfaChainTypes {
    type Components: OfaComponents<Self>;

    // Host methods

    fn host_height(&self) -> Self::Height;

    fn host_timestamp(&self) -> Self::Timestamp;

    fn add_duration(time: &Self::Timestamp, duration: &Self::Duration) -> Self::Timestamp;

    // Message methods

    fn message_type(message: &Self::Message) -> &Self::MessageType;

    fn message_signer(message: &Self::Message) -> &Self::Signer;

    // AnyClientMethods

    fn client_state_type(client_state: &Self::AnyClientState) -> Self::ClientType;

    fn client_state_is_frozen(client_state: &Self::AnyClientState) -> bool;

    fn client_state_trusting_period(client_state: &Self::AnyClientState) -> Self::Duration;

    fn client_state_latest_height(client_state: &Self::AnyClientState) -> Self::Height;

    fn consensus_state_timestamp(consensus_state: &Self::AnyConsensusState) -> Self::Timestamp;

    fn client_header_height(client_header: &Self::AnyClientHeader) -> Self::Height;

    // AnyClientReader methods

    fn get_client_type(&self, client_id: &Self::ClientId) -> Result<Self::ClientType, Self::Error>;

    fn get_any_client_state(
        &self,
        client_id: &Self::ClientId,
    ) -> Result<Self::AnyClientState, Self::Error>;

    fn get_latest_any_consensus_state(
        &self,
        client_id: &Self::ClientId,
    ) -> Result<Self::AnyConsensusState, Self::Error>;

    fn get_any_consensus_state_at_height(
        &self,
        client_id: &Self::ClientId,
        height: &Self::Height,
    ) -> Result<Option<Self::AnyConsensusState>, Self::Error>;

    fn get_any_consensus_state_after_height(
        &self,
        client_id: &Self::ClientId,
        height: &Self::Height,
    ) -> Result<Option<Self::AnyConsensusState>, Self::Error>;

    fn get_any_consensus_state_before_height(
        &self,
        client_id: &Self::ClientId,
        height: &Self::Height,
    ) -> Result<Option<Self::AnyConsensusState>, Self::Error>;

    // AnyClientWriter methods

    fn set_any_client_state(
        &self,
        client_id: &Self::ClientId,
        client_state: &Self::AnyClientState,
    ) -> Result<(), Self::Error>;

    fn set_any_consensus_state(
        &self,
        client_id: &Self::ClientId,
        consensus_state: &Self::AnyConsensusState,
    ) -> Result<(), Self::Error>;

    // Error methods

    fn client_type_mismatch_error(expected_client_type: &Self::ClientType) -> Self::Error;

    fn unknown_message_error(message_type: &Self::MessageType) -> Self::Error;

    fn client_frozen_error(client_id: &Self::ClientId) -> Self::Error;

    fn client_expired_error(
        client_id: &Self::ClientId,
        current_time: &Self::Timestamp,
        latest_allowed_update_time: &Self::Timestamp,
    ) -> Self::Error;

    // Event methods

    fn update_client_event(
        client_id: &Self::ClientId,
        client_type: &Self::ClientType,
        consensus_height: &Self::Height,
        header: &Self::AnyClientHeader,
    ) -> Self::Event;

    fn misbehavior_event(
        client_id: &Self::ClientId,
        client_type: &Self::ClientType,
        consensus_height: &Self::Height,
        header: &Self::AnyClientHeader,
    ) -> Self::Event;
}
