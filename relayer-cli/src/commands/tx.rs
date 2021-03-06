//! `tx` subcommand
use abscissa_core::{Command, Help, Options, Runnable};

use crate::commands::tx::client::{TxCreateClientCmd, TxUpdateClientCmd};

mod channel;
mod client;
mod connection;

/// `tx` subcommand
#[derive(Command, Debug, Options, Runnable)]
pub enum TxCmd {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `tx raw` subcommand
    #[options(help = "tx raw")]
    Raw(TxRawCommands),
}

#[derive(Command, Debug, Options, Runnable)]
pub enum TxRawCommands {
    /// The `help` subcommand
    #[options(help = "get usage information")]
    Help(Help<Self>),

    /// The `tx raw client-create` subcommand submits a MsgCreateClient in a transaction to a chain
    #[options(help = "tx raw create-client")]
    CreateClient(TxCreateClientCmd),

    /// The `tx raw client-update` subcommand submits a MsgUpdateClient in a transaction to a chain
    #[options(help = "tx raw update-client")]
    UpdateClient(TxUpdateClientCmd),

    /// The `tx raw conn-init` subcommand
    #[options(help = "tx raw conn-init")]
    ConnInit(connection::TxRawConnInitCmd),

    /// The `tx raw conn-try` subcommand
    #[options(help = "tx raw conn-try")]
    ConnTry(connection::TxRawConnTryCmd),

    /// The `tx raw conn-ack` subcommand
    #[options(help = "tx raw conn-ack")]
    ConnAck(connection::TxRawConnAckCmd),

    /// The `tx raw conn-confirm` subcommand
    #[options(help = "tx raw conn-confirm")]
    ConnConfirm(connection::TxRawConnConfirmCmd),

    /// The `tx raw chan-init` subcommand
    #[options(help = "tx raw chan-init")]
    ChanInit(channel::TxRawChanInitCmd),

    /// The `tx raw chan-try` subcommand
    #[options(help = "tx raw chan-try")]
    ChanTry(channel::TxRawChanTryCmd),

    /// The `tx raw chan-ack` subcommand
    #[options(help = "tx raw chan-ack")]
    ChanAck(channel::TxRawChanAckCmd),

    /// The `tx raw chan-confirm` subcommand
    #[options(help = "tx raw chan-confirm")]
    ChanConfirm(channel::TxRawChanConfirmCmd),
}
