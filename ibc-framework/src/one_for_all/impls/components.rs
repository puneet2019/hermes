use crate::core::traits::handlers::update_client::HasAnyUpdateClientHandler;
use crate::core::traits::stores::client_reader::HasAnyClientReader;
use crate::core::traits::stores::client_writer::HasAnyClientWriter;
use crate::one_for_all::traits::chain::OfaChain;
use crate::one_for_all::traits::components::{OfaChainComponents, OfaClientComponents};
use crate::one_for_all::types::chain::OfaChainWrapper;

impl<Chain> HasAnyClientReader for OfaChainWrapper<Chain>
where
    Chain: OfaChain,
{
    type AnyClientReader = <Chain::ChainComponents as OfaChainComponents<Chain>>::AnyClientReader;
}

impl<Chain> HasAnyClientWriter for OfaChainWrapper<Chain>
where
    Chain: OfaChain,
{
    type AnyClientWriter = <Chain::ChainComponents as OfaChainComponents<Chain>>::AnyClientWriter;
}

impl<Chain> HasAnyUpdateClientHandler for OfaChainWrapper<Chain>
where
    Chain: OfaChain,
{
    type AnyUpdateClientHandler =
        <Chain::ClientComponents as OfaClientComponents<Chain>>::AnyUpdateClientHandler;
}