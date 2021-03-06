use crate::ics04_channel::error::{self, Error, Kind};
use crate::ics24_host::identifier::{ChannelId, ConnectionId, PortId};

use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
use ibc_proto::ibc::core::channel::v1::Counterparty as RawCounterparty;

use tendermint_proto::Protobuf;

use anomaly::fail;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct ChannelEnd {
    state: State,
    ordering: Order,
    remote: Counterparty,
    connection_hops: Vec<ConnectionId>,
    version: String,
}

impl Protobuf<RawChannel> for ChannelEnd {}

impl TryFrom<RawChannel> for ChannelEnd {
    type Error = anomaly::Error<Kind>;

    fn try_from(value: RawChannel) -> Result<Self, Self::Error> {
        // Parse the ordering type. Propagate the error, if any, to our caller.
        let chan_ordering = Order::from_i32(value.ordering)?;

        let chan_state = State::from_i32(value.state)?;

        // Assemble the 'remote' attribute of the Channel, which represents the Counterparty.
        let remote = value
            .counterparty
            .ok_or(Kind::MissingCounterparty)?
            .try_into()?;

        // Parse each item in connection_hops into a ConnectionId.
        let connection_hops = value
            .connection_hops
            .into_iter()
            .map(|conn_id| ConnectionId::from_str(conn_id.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Kind::IdentifierError.context(e))?;

        let version = validate_version(value.version)?;

        Ok(ChannelEnd::new(
            chan_state,
            chan_ordering,
            remote,
            connection_hops,
            version,
        ))
    }
}

impl From<ChannelEnd> for RawChannel {
    fn from(value: ChannelEnd) -> Self {
        RawChannel {
            state: value.state.clone() as i32,
            ordering: value.ordering as i32,
            counterparty: Some(value.counterparty().into()),
            connection_hops: value
                .connection_hops
                .iter()
                .map(|v| v.as_str().to_string())
                .collect(),
            version: value.version,
        }
    }
}

impl ChannelEnd {
    /// Creates a new ChannelEnd in state Uninitialized and other fields parametrized.
    pub fn new(
        state: State,
        ordering: Order,
        remote: Counterparty,
        connection_hops: Vec<ConnectionId>,
        version: String,
    ) -> Self {
        Self {
            state,
            ordering,
            remote,
            connection_hops,
            version,
        }
    }

    /// Updates the ChannelEnd to assume a new State 's'.
    pub fn set_state(&mut self, s: State) {
        self.state = s;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn ordering(&self) -> &Order {
        &self.ordering
    }

    pub fn counterparty(&self) -> Counterparty {
        self.remote.clone()
    }

    pub fn connection_hops(&self) -> Vec<ConnectionId> {
        self.connection_hops.clone()
    }

    pub fn version(&self) -> String {
        self.version.parse().unwrap()
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        if self.connection_hops.len() != 1 {
            return Err(Kind::InvalidConnectionHopsLength
                .context("validate channel")
                .into());
        }
        if self.version().trim() == "" {
            return Err(Kind::InvalidVersion.context("empty version string").into());
        }
        self.counterparty().validate_basic()
    }

    /// Helper function to compare the state of this end with another state.
    pub fn state_matches(&self, other: &State) -> bool {
        self.state.eq(other)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Counterparty {
    pub port_id: PortId,
    pub channel_id: Option<ChannelId>,
}

impl Counterparty {
    pub fn new(port_id: PortId, channel_id: Option<ChannelId>) -> Self {
        Self {
            port_id,
            channel_id,
        }
    }

    pub fn port_id(&self) -> &PortId {
        &self.port_id
    }

    pub fn channel_id(&self) -> Option<&ChannelId> {
        self.channel_id.as_ref()
    }

    pub fn validate_basic(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl Protobuf<RawCounterparty> for Counterparty {}

impl TryFrom<RawCounterparty> for Counterparty {
    type Error = anomaly::Error<Kind>;

    fn try_from(value: RawCounterparty) -> Result<Self, Self::Error> {
        let channel_id = Some(value.channel_id)
            .filter(|x| !x.is_empty())
            .map(|v| FromStr::from_str(v.as_str()))
            .transpose()
            .map_err(|e| Kind::IdentifierError.context(e))?;
        Ok(Counterparty::new(
            value
                .port_id
                .parse()
                .map_err(|e| Kind::IdentifierError.context(e))?,
            channel_id,
        ))
    }
}

impl From<Counterparty> for RawCounterparty {
    fn from(value: Counterparty) -> Self {
        RawCounterparty {
            port_id: value.port_id.as_str().to_string(),
            channel_id: value
                .channel_id
                .map_or_else(|| "".to_string(), |v| v.as_str().to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Order {
    None = 0,
    Unordered,
    Ordered,
}

impl Default for Order {
    fn default() -> Self {
        Order::Unordered
    }
}

impl Order {
    /// Yields the Order as a string
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::None => "UNINITIALIZED",
            Self::Unordered => "UNORDERED",
            Self::Ordered => "ORDERED",
        }
    }

    // Parses the Order out from a i32.
    pub fn from_i32(nr: i32) -> Result<Self, Error> {
        match nr {
            0 => Ok(Self::None),
            1 => Ok(Self::Unordered),
            2 => Ok(Self::Ordered),
            _ => fail!(error::Kind::UnknownOrderType, nr),
        }
    }
}

impl FromStr for Order {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNINITIALIZED" => Ok(Self::None),
            "UNORDERED" => Ok(Self::Unordered),
            "ORDERED" => Ok(Self::Ordered),
            _ => fail!(error::Kind::UnknownOrderType, s),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Init = 1,
    TryOpen = 2,
    Open = 3,
    Closed = 4,
}

impl State {
    /// Yields the state as a string
    pub fn as_string(&self) -> &'static str {
        match self {
            Self::Init => "INIT",
            Self::TryOpen => "TRYOPEN",
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
        }
    }

    // Parses the State out from a i32.
    pub fn from_i32(s: i32) -> Result<Self, Error> {
        match s {
            1 => Ok(Self::Init),
            2 => Ok(Self::TryOpen),
            3 => Ok(Self::Open),
            4 => Ok(Self::Closed),
            _ => fail!(error::Kind::UnknownState, s),
        }
    }
}

/// Version validation, specific for channel (ICS4) opening handshake protocol.
/// This field is supposed to be opaque to the core IBC protocol. No explicit validation necessary,
/// and empty version is currently allowed by the specification (cf. ICS 004, v1).
pub fn validate_version(version: String) -> Result<String, Error> {
    Ok(version)
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
    use ibc_proto::ibc::core::channel::v1::Counterparty as RawCounterparty;

    /// Returns a dummy `RawCounterparty`, for testing only!
    pub fn get_dummy_raw_counterparty() -> RawCounterparty {
        RawCounterparty {
            port_id: "0123456789".into(),
            channel_id: "0987654321".into(),
        }
    }

    /// Returns a dummy `RawChannel`, for testing only!
    pub fn get_dummy_raw_channel_end() -> RawChannel {
        RawChannel {
            state: 1,
            ordering: 0,
            counterparty: Some(get_dummy_raw_counterparty()),
            connection_hops: vec![],
            version: "".to_string(), // The version is not validated.
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::ics04_channel::channel::test_util::get_dummy_raw_channel_end;
    use crate::ics04_channel::channel::ChannelEnd;

    use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
    use std::convert::TryFrom;

    #[test]
    fn channel_end_try_from_raw() {
        let raw_channel_end = get_dummy_raw_channel_end();

        let empty_raw_channel_end = RawChannel {
            counterparty: None,
            ..raw_channel_end.clone()
        };

        struct Test {
            name: String,
            params: RawChannel,
            want_pass: bool,
        }

        let tests: Vec<Test> = vec![
            Test {
                name: "Raw channel end with missing counterparty".to_string(),
                params: empty_raw_channel_end,
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect state".to_string(),
                params: RawChannel {
                    state: -1,
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect ordering".to_string(),
                params: RawChannel {
                    ordering: -1,
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect connection id in connection hops".to_string(),
                params: RawChannel {
                    connection_hops: vec!["connection*".to_string()].into_iter().collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with incorrect connection id (has blank space)".to_string(),
                params: RawChannel {
                    connection_hops: vec!["con nection".to_string()].into_iter().collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Raw channel end with two correct connection ids in connection hops"
                    .to_string(),
                params: RawChannel {
                    connection_hops: vec!["connection1".to_string(), "connection2".to_string()]
                        .into_iter()
                        .collect(),
                    ..raw_channel_end.clone()
                },
                want_pass: true,
            },
            Test {
                name: "Raw channel end with correct params".to_string(),
                params: raw_channel_end,
                want_pass: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            let p = test.params.clone();

            let ce_result = ChannelEnd::try_from(p);

            assert_eq!(
                test.want_pass,
                ce_result.is_ok(),
                "ChannelEnd::try_from() failed for test {}, \nmsg{:?} with error {:?}",
                test.name,
                test.params.clone(),
                ce_result.err(),
            );
        }
    }

    #[test]
    fn parse_channel_ordering_type() {
        use super::Order;

        struct Test {
            ordering: &'static str,
            want_res: Order,
            want_err: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                ordering: "UNINITIALIZED",
                want_res: Order::None,
                want_err: false,
            },
            Test {
                ordering: "UNORDERED",
                want_res: Order::Unordered,
                want_err: false,
            },
            Test {
                ordering: "ORDERED",
                want_res: Order::Ordered,
                want_err: false,
            },
            Test {
                ordering: "UNKNOWN_ORDER",
                want_res: Order::None,
                want_err: true,
            },
        ]
        .into_iter()
        .collect();

        for test in tests {
            match Order::from_str(test.ordering) {
                Ok(res) => {
                    assert!(!test.want_err);
                    assert_eq!(test.want_res, res);
                }
                Err(_) => assert!(test.want_err, "parse failed"),
            }
        }
    }
}
