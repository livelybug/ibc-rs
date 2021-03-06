use std::convert::{TryFrom, TryInto};

use tendermint::account::Id as AccountId;
use tendermint_proto::Protobuf;

use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;

use crate::address::{account_to_string, string_to_account};
use crate::ics04_channel::error::{Error, Kind};
use crate::ics04_channel::packet::Packet;
use crate::ics23_commitment::commitment::CommitmentProof;
use crate::{proofs::Proofs, tx_msg::Msg, Height};

/// Message type for `MsgPacket`.
const TYPE_MSG_PACKET: &str = "ics04/opaque";

///
/// Message definition for the "packet receiving" datagram.
///
#[derive(Clone, Debug, PartialEq)]
pub struct MsgRecvPacket {
    packet: Packet,
    proofs: Proofs,
    signer: AccountId,
}

impl MsgRecvPacket {
    // todo: Constructor not used yet.
    #[allow(dead_code, unreachable_code, unused_variables)]
    fn new(
        packet: Packet,
        proof: CommitmentProof,
        proof_height: Height,
        signer: AccountId,
    ) -> Result<MsgRecvPacket, Error> {
        Ok(Self {
            packet: todo!(),
            proofs: Proofs::new(proof, None, None, proof_height)
                .map_err(|e| Kind::InvalidProof.context(e))?,
            signer,
        })
    }

    // returns the base64-encoded bytes used for the
    // data field when signing the packet
    pub fn get_data_bytes() -> Vec<u8> {
        todo!()
    }
}

impl Msg for MsgRecvPacket {
    type ValidationError = Error;

    fn route(&self) -> String {
        crate::keys::ROUTER_KEY.to_string()
    }

    fn get_type(&self) -> String {
        TYPE_MSG_PACKET.to_string()
    }

    fn validate_basic(&self) -> Result<(), Self::ValidationError> {
        // Nothing to validate
        // All the validation is performed on creation
        Ok(())
    }

    fn get_signers(&self) -> Vec<AccountId> {
        vec![self.signer]
    }
}

impl Protobuf<RawMsgRecvPacket> for MsgRecvPacket {}

impl TryFrom<RawMsgRecvPacket> for MsgRecvPacket {
    type Error = anomaly::Error<Kind>;

    fn try_from(raw_msg: RawMsgRecvPacket) -> Result<Self, Self::Error> {
        let signer =
            string_to_account(raw_msg.signer).map_err(|e| Kind::InvalidSigner.context(e))?;

        let proofs = Proofs::new(
            raw_msg.proof.into(),
            None,
            None,
            raw_msg
                .proof_height
                .ok_or(Kind::MissingHeight)?
                .try_into()
                .map_err(|e| Kind::InvalidProof.context(e))?,
        )
        .map_err(|e| Kind::InvalidProof.context(e))?;

        Ok(MsgRecvPacket {
            packet: raw_msg
                .packet
                .ok_or(Kind::MissingPacket)?
                .try_into()
                .map_err(|e| Kind::InvalidPacket.context(e))?,
            proofs,
            signer,
        })
    }
}

impl From<MsgRecvPacket> for RawMsgRecvPacket {
    fn from(domain_msg: MsgRecvPacket) -> Self {
        RawMsgRecvPacket {
            packet: Some(domain_msg.packet.into()),
            proof: domain_msg.proofs.object_proof().clone().into(),
            proof_height: Some(domain_msg.proofs.height().into()),
            signer: account_to_string(domain_msg.signer).unwrap(),
        }
    }
}

#[cfg(test)]
mod test_util {
    use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;
    use ibc_proto::ibc::core::client::v1::Height as RawHeight;

    use crate::ics04_channel::packet::test_utils::get_dummy_raw_packet;
    use crate::test_utils::{get_dummy_bech32_account, get_dummy_proof};

    /// Returns a dummy `RawMsgRecvPacket`, for testing only! The `height` parametrizes both the
    /// proof height as well as the timeout height.
    pub fn get_dummy_raw_msg_recv_packet(height: u64) -> RawMsgRecvPacket {
        RawMsgRecvPacket {
            packet: Some(get_dummy_raw_packet(height)),
            proof: get_dummy_proof(),
            proof_height: Some(RawHeight {
                version_number: 0,
                version_height: height,
            }),
            signer: get_dummy_bech32_account(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::convert::{TryFrom, TryInto};

    use ibc_proto::ibc::core::channel::v1::MsgRecvPacket as RawMsgRecvPacket;

    use crate::ics04_channel::error::Error;
    use crate::ics04_channel::msgs::recv_packet::test_util::get_dummy_raw_msg_recv_packet;
    use crate::ics04_channel::msgs::recv_packet::MsgRecvPacket;

    #[test]
    fn msg_recv_packet_try_from_raw() {
        struct Test {
            name: String,
            raw: RawMsgRecvPacket,
            want_pass: bool,
        }

        let height = 20;
        let default_raw_msg = get_dummy_raw_msg_recv_packet(height);
        let tests: Vec<Test> = vec![
            Test {
                name: "Good parameters".to_string(),
                raw: default_raw_msg.clone(),
                want_pass: true,
            },
            Test {
                name: "Missing proof".to_string(),
                raw: RawMsgRecvPacket {
                    proof: vec![],
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing proof height".to_string(),
                raw: RawMsgRecvPacket {
                    proof_height: None,
                    ..default_raw_msg.clone()
                },
                want_pass: false,
            },
            Test {
                name: "Missing signer".to_string(),
                raw: RawMsgRecvPacket {
                    signer: "".to_string(),
                    ..default_raw_msg
                },
                want_pass: false,
            },
        ];

        for test in tests {
            let res_msg: Result<MsgRecvPacket, Error> = test.raw.clone().try_into();

            assert_eq!(
                res_msg.is_ok(),
                test.want_pass,
                "MsgRecvPacket::try_from failed for test {} \nraw message: {:?} with error: {:?}",
                test.name,
                test.raw,
                res_msg.err()
            );
        }
    }

    #[test]
    fn to_and_from() {
        let raw = get_dummy_raw_msg_recv_packet(15);
        let msg = MsgRecvPacket::try_from(raw.clone()).unwrap();
        let raw_back = RawMsgRecvPacket::from(msg.clone());
        let msg_back = MsgRecvPacket::try_from(raw_back.clone()).unwrap();
        assert_eq!(raw, raw_back);
        assert_eq!(msg, msg_back);
    }
}
