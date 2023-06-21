use core::time::Duration;
use eyre::eyre;
use ibc_proto::ibc::core::channel::v1::FlushStatus;
use ibc_relayer::chain::handle::ChainHandle;
use ibc_relayer::chain::requests::{IncludeProof, QueryChannelRequest, QueryHeight};
use ibc_relayer::channel::{extract_channel_id, Channel, ChannelSide};
use ibc_relayer_types::core::ics04_channel::channel::{
    ChannelEnd, IdentifiedChannelEnd, Ordering, State as ChannelState,
};
use ibc_relayer_types::core::ics04_channel::version::Version;
use ibc_relayer_types::core::ics24_host::identifier::ConnectionId;

use crate::error::Error;
use crate::types::id::{
    TaggedChannelId, TaggedChannelIdRef, TaggedClientIdRef, TaggedConnectionIdRef, TaggedPortId,
    TaggedPortIdRef,
};
use crate::types::tagged::DualTagged;
use crate::util::retry::assert_eventually_succeed;

pub trait TaggedChannelEndExt<ChainA, ChainB> {
    fn tagged_counterparty_channel_id(&self) -> Option<TaggedChannelId<ChainB, ChainA>>;
    fn tagged_counterparty_port_id(&self) -> TaggedPortId<ChainB, ChainA>;
}

impl<ChainA, ChainB> TaggedChannelEndExt<ChainA, ChainB>
    for DualTagged<ChainA, ChainB, ChannelEnd>
{
    fn tagged_counterparty_channel_id(&self) -> Option<TaggedChannelId<ChainB, ChainA>> {
        self.contra_map(|c| c.counterparty().channel_id.clone())
            .transpose()
    }

    fn tagged_counterparty_port_id(&self) -> TaggedPortId<ChainB, ChainA> {
        self.contra_map(|c| c.counterparty().port_id.clone())
    }
}

/// This struct contains the attributes which can be modified with a channel upgrade
pub struct ChannelUpgradableAttributes {
    version: Version,
    ordering: Ordering,
    connection_hops_a: Vec<ConnectionId>,
    connection_hops_b: Vec<ConnectionId>,
}

impl ChannelUpgradableAttributes {
    pub fn new(
        version: Version,
        ordering: Ordering,
        connection_hops_a: Vec<ConnectionId>,
        connection_hops_b: Vec<ConnectionId>,
    ) -> Self {
        Self {
            version,
            ordering,
            connection_hops_a,
            connection_hops_b,
        }
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn ordering(&self) -> &Ordering {
        &self.ordering
    }

    pub fn connection_hops_a(&self) -> &Vec<ConnectionId> {
        &self.connection_hops_a
    }

    pub fn connection_hops_b(&self) -> &Vec<ConnectionId> {
        &self.connection_hops_b
    }
}

pub fn init_channel<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    client_id_a: &TaggedClientIdRef<ChainA, ChainB>,
    client_id_b: &TaggedClientIdRef<ChainB, ChainA>,
    connection_id_a: &TaggedConnectionIdRef<ChainA, ChainB>,
    connection_id_b: &TaggedConnectionIdRef<ChainB, ChainA>,
    src_port_id: &TaggedPortIdRef<ChainA, ChainB>,
    dst_port_id: &TaggedPortIdRef<ChainB, ChainA>,
) -> Result<(TaggedChannelId<ChainB, ChainA>, Channel<ChainB, ChainA>), Error> {
    let channel = Channel {
        connection_delay: Default::default(),
        ordering: Ordering::Unordered,
        a_side: ChannelSide::new(
            handle_a.clone(),
            client_id_a.cloned_value(),
            connection_id_a.cloned_value(),
            src_port_id.cloned_value(),
            None,
            None,
        ),
        b_side: ChannelSide::new(
            handle_b.clone(),
            client_id_b.cloned_value(),
            connection_id_b.cloned_value(),
            dst_port_id.cloned_value(),
            None,
            None,
        ),
    };

    let event = channel.build_chan_open_init_and_send()?;
    let channel_id = extract_channel_id(&event)?.clone();
    let channel2 = Channel::restore_from_event(handle_b.clone(), handle_a.clone(), event)?;

    Ok((DualTagged::new(channel_id), channel2))
}

pub fn init_channel_optimistic<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    client_id_a: &TaggedClientIdRef<ChainA, ChainB>,
    client_id_b: &TaggedClientIdRef<ChainB, ChainA>,
    connection_id_b: &TaggedConnectionIdRef<ChainB, ChainA>,
    src_port_id: &TaggedPortIdRef<ChainA, ChainB>,
    dst_port_id: &TaggedPortIdRef<ChainB, ChainA>,
) -> Result<TaggedChannelId<ChainB, ChainA>, Error> {
    let channel = Channel {
        connection_delay: Default::default(),
        ordering: Ordering::Unordered,
        a_side: ChannelSide::new(
            handle_a.clone(),
            client_id_a.cloned_value(),
            ConnectionId::default(),
            src_port_id.cloned_value(),
            None,
            None,
        ),
        b_side: ChannelSide::new(
            handle_b.clone(),
            client_id_b.cloned_value(),
            connection_id_b.cloned_value(),
            dst_port_id.cloned_value(),
            None,
            None,
        ),
    };

    let event = channel.build_chan_open_init_and_send()?;
    let channel_id = extract_channel_id(&event)?.clone();
    Ok(DualTagged::new(channel_id))
}

pub fn try_channel<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel: &Channel<ChainB, ChainA>,
) -> Result<(TaggedChannelId<ChainA, ChainB>, Channel<ChainA, ChainB>), Error> {
    let event = channel.build_chan_open_try_and_send()?;
    let channel_id = extract_channel_id(&event)?.clone();
    let channel2 = Channel::restore_from_event(handle_a.clone(), handle_b.clone(), event)?;

    Ok((DualTagged::new(channel_id), channel2))
}

pub fn ack_channel<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel: &Channel<ChainB, ChainA>,
) -> Result<(TaggedChannelId<ChainA, ChainB>, Channel<ChainA, ChainB>), Error> {
    let event = channel.build_chan_open_ack_and_send()?;
    let channel_id = extract_channel_id(&event)?.clone();
    let channel2 = Channel::restore_from_event(handle_a.clone(), handle_b.clone(), event)?;

    Ok((DualTagged::new(channel_id), channel2))
}

pub fn query_channel_end<ChainA: ChainHandle, ChainB>(
    handle: &ChainA,
    channel_id: &TaggedChannelIdRef<ChainA, ChainB>,
    port_id: &TaggedPortIdRef<ChainA, ChainB>,
) -> Result<DualTagged<ChainA, ChainB, ChannelEnd>, Error> {
    let (channel_end, _) = handle.query_channel(
        QueryChannelRequest {
            port_id: port_id.into_value().clone(),
            channel_id: channel_id.into_value().clone(),
            height: QueryHeight::Latest,
        },
        IncludeProof::Yes,
    )?;

    Ok(DualTagged::new(channel_end))
}

pub fn query_identified_channel_end<ChainA: ChainHandle, ChainB>(
    handle: &ChainA,
    channel_id: TaggedChannelIdRef<ChainA, ChainB>,
    port_id: TaggedPortIdRef<ChainA, ChainB>,
) -> Result<DualTagged<ChainA, ChainB, IdentifiedChannelEnd>, Error> {
    let (channel_end, _) = handle.query_channel(
        QueryChannelRequest {
            port_id: port_id.into_value().clone(),
            channel_id: channel_id.into_value().clone(),
            height: QueryHeight::Latest,
        },
        IncludeProof::No,
    )?;
    Ok(DualTagged::new(IdentifiedChannelEnd::new(
        port_id.into_value().clone(),
        channel_id.into_value().clone(),
        channel_end,
    )))
}

pub fn assert_eventually_channel_established<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel_id_a: &TaggedChannelIdRef<ChainA, ChainB>,
    port_id_a: &TaggedPortIdRef<ChainA, ChainB>,
) -> Result<TaggedChannelId<ChainB, ChainA>, Error> {
    assert_eventually_succeed(
        "channel should eventually established",
        20,
        Duration::from_secs(1),
        || {
            let channel_end_a = query_channel_end(handle_a, channel_id_a, port_id_a)?;

            if !channel_end_a.value().state_matches(&ChannelState::Open) {
                return Err(Error::generic(eyre!(
                    "expected channel end A to be in open state"
                )));
            }

            let channel_id_b = channel_end_a
                .tagged_counterparty_channel_id()
                .ok_or_else(|| {
                    eyre!("expected counterparty channel id to present on open channel")
                })?;

            let port_id_b = channel_end_a.tagged_counterparty_port_id();

            let channel_end_b =
                query_channel_end(handle_b, &channel_id_b.as_ref(), &port_id_b.as_ref())?;

            if !channel_end_b.value().state_matches(&ChannelState::Open) {
                return Err(Error::generic(eyre!(
                    "expected channel end B to be in open state"
                )));
            }

            Ok(channel_id_b)
        },
    )
}

pub fn assert_eventually_channel_upgrade_init<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel_id_a: &TaggedChannelIdRef<ChainA, ChainB>,
    port_id_a: &TaggedPortIdRef<ChainA, ChainB>,
    upgrade_attrs: &ChannelUpgradableAttributes,
) -> Result<TaggedChannelId<ChainB, ChainA>, Error> {
    assert_eventually_succeed(
        "channel upgrade should be initialised",
        20,
        Duration::from_secs(1),
        || {
            assert_eventually_succeed(
                "channel upgrade should be initialised",
                20,
                Duration::from_secs(1),
                || {
                    assert_channel_upgrade_state(
                        ChannelState::InitUpgrade,
                        ChannelState::Open,
                        FlushStatus::NotinflushUnspecified,
                        FlushStatus::NotinflushUnspecified,
                        handle_a,
                        handle_b,
                        channel_id_a,
                        port_id_a,
                        upgrade_attrs,
                    )
                },
            )
        },
    )
}

pub fn assert_eventually_channel_upgrade_try<ChainA: ChainHandle, ChainB: ChainHandle>(
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel_id_a: &TaggedChannelIdRef<ChainA, ChainB>,
    port_id_a: &TaggedPortIdRef<ChainA, ChainB>,
    upgrade_attrs: &ChannelUpgradableAttributes,
) -> Result<TaggedChannelId<ChainB, ChainA>, Error> {
    assert_eventually_succeed(
        "channel upgrade should be initialised",
        20,
        Duration::from_secs(1),
        || {
            assert_channel_upgrade_state(
                ChannelState::InitUpgrade,
                ChannelState::TryUpgrade,
                FlushStatus::NotinflushUnspecified,
                FlushStatus::Flushcomplete,
                handle_a,
                handle_b,
                channel_id_a,
                port_id_a,
                upgrade_attrs,
            )
        },
    )
}

fn assert_channel_upgrade_state<ChainA: ChainHandle, ChainB: ChainHandle>(
    a_side_state: ChannelState,
    b_side_state: ChannelState,
    a_side_flush_status: FlushStatus,
    b_side_flush_status: FlushStatus,
    handle_a: &ChainA,
    handle_b: &ChainB,
    channel_id_a: &TaggedChannelIdRef<ChainA, ChainB>,
    port_id_a: &TaggedPortIdRef<ChainA, ChainB>,
    upgrade_attrs: &ChannelUpgradableAttributes,
) -> Result<TaggedChannelId<ChainB, ChainA>, Error> {
    let channel_end_a = query_channel_end(handle_a, channel_id_a, port_id_a)?;

    if !channel_end_a.value().state_matches(&a_side_state) {
        return Err(Error::generic(eyre!(
            "expected channel end A state to be `{}`, but is instead `{}`",
            a_side_state,
            channel_end_a.value().state()
        )));
    }

    if !channel_end_a
        .value()
        .flush_status_matches(&a_side_flush_status)
    {
        return Err(Error::generic(eyre!(
            "expected channel end A flush status to be `{}`, but is instead `{}`",
            a_side_flush_status.as_str_name(),
            channel_end_a.value().flush_status().as_str_name()
        )));
    }

    if !channel_end_a
        .value()
        .version_matches(upgrade_attrs.version())
    {
        return Err(Error::generic(eyre!(
            "expected channel end A version to be `{}`, but it is instead `{}`",
            upgrade_attrs.version(),
            channel_end_a.value().version()
        )));
    }

    if !channel_end_a
        .value()
        .order_matches(upgrade_attrs.ordering())
    {
        return Err(Error::generic(eyre!(
            "expected channel end A ordering to be `{}`, but it is instead `{}`",
            upgrade_attrs.ordering(),
            channel_end_a.value().ordering()
        )));
    }

    if !channel_end_a
        .value()
        .connection_hops_matches(upgrade_attrs.connection_hops_a())
    {
        return Err(Error::generic(eyre!(
            "expected channel end A connection hops to be `{:?}`, but it is instead `{:?}`",
            upgrade_attrs.connection_hops_a(),
            channel_end_a.value().connection_hops()
        )));
    }

    let channel_id_b = channel_end_a
        .tagged_counterparty_channel_id()
        .ok_or_else(|| eyre!("expected counterparty channel id to present on open channel"))?;

    let port_id_b = channel_end_a.tagged_counterparty_port_id();

    let channel_end_b = query_channel_end(handle_b, &channel_id_b.as_ref(), &port_id_b.as_ref())?;

    if !channel_end_b.value().state_matches(&b_side_state) {
        return Err(Error::generic(eyre!(
            "expected channel end B state to be `{}`, but is instead `{}`",
            b_side_state,
            channel_end_b.value().state()
        )));
    }

    if !channel_end_b
        .value()
        .flush_status_matches(&b_side_flush_status)
    {
        return Err(Error::generic(eyre!(
            "expected channel end B flush status to be `{}`, but is instead `{}`",
            b_side_flush_status.as_str_name(),
            channel_end_b.value().flush_status().as_str_name()
        )));
    }

    if !channel_end_b
        .value()
        .version_matches(upgrade_attrs.version())
    {
        return Err(Error::generic(eyre!(
            "expected channel end B version to be `{}`, but it is instead `{}`",
            upgrade_attrs.version(),
            channel_end_b.value().version()
        )));
    }

    if !channel_end_b
        .value()
        .order_matches(upgrade_attrs.ordering())
    {
        return Err(Error::generic(eyre!(
            "expected channel end B ordering to be `{}`, but it is instead `{}`",
            upgrade_attrs.ordering(),
            channel_end_b.value().ordering()
        )));
    }

    if !channel_end_b
        .value()
        .connection_hops_matches(upgrade_attrs.connection_hops_b())
    {
        return Err(Error::generic(eyre!(
            "expected channel end B connection hops to be `{:?}`, but it is instead `{:?}`",
            upgrade_attrs.connection_hops_b(),
            channel_end_b.value().connection_hops()
        )));
    }

    Ok(channel_id_b)
}
