use crate::base::one_for_all::traits::chain::OfaChainContext;
use crate::base::one_for_all::traits::telemetry::OfaTelemetryWrapper;
use crate::base::traits::contexts::telemetry::HasTelemetry;
use crate::full::one_for_all::traits::chain::OfaFullChain;

impl<Chain: OfaFullChain> HasTelemetry for OfaChainContext<Chain> {
    type Telemetry = OfaTelemetryWrapper<Chain::Telemetry>;

    fn telemetry(&self) -> &Self::Telemetry {
        self.chain.telemetry()
    }
}