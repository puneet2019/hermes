use ibc_relayer::chain::handle::ChainHandle;
use ibc_relayer_cosmos::cosmos::all_for_one::relay::AfoCosmosRelayContext;
use ibc_relayer_cosmos::cosmos::batch::chain::CosmosChainEnv;
use ibc_relayer_cosmos::cosmos::batch::relay::new_relay_context_with_batch;
use ibc_relayer_runtime::tokio::context::TokioRuntimeContext;
use ibc_test_framework::types::binary::chains::ConnectedChains;

pub fn build_cosmos_relay_context<ChainA, ChainB>(
    chains: &ConnectedChains<ChainA, ChainB>,
) -> impl AfoCosmosRelayContext
where
    ChainA: ChainHandle,
    ChainB: ChainHandle,
{
    let runtime = TokioRuntimeContext::new(chains.node_a.value().chain_driver.runtime.clone());

    let chain_a = CosmosChainEnv::new(
        chains.handle_a.clone(),
        chains
            .node_a
            .value()
            .wallets
            .relayer
            .address
            .0
            .parse()
            .unwrap(),
        chains.node_a.value().chain_driver.tx_config.clone(),
        chains.node_a.value().wallets.relayer.key.clone(),
    );

    let chain_b = CosmosChainEnv::new(
        chains.handle_b.clone(),
        chains
            .node_b
            .value()
            .wallets
            .relayer
            .address
            .0
            .parse()
            .unwrap(),
        chains.node_b.value().chain_driver.tx_config.clone(),
        chains.node_b.value().wallets.relayer.key.clone(),
    );

    let relay = new_relay_context_with_batch(
        runtime,
        chain_a,
        chain_b,
        chains.foreign_clients.client_a_to_b.clone(),
        chains.foreign_clients.client_b_to_a.clone(),
        Default::default(),
    );

    relay
}
