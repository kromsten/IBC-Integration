use crate::cw_types::{
    CwChannelCloseMsg, CwChannelConnectMsg, CwChannelOpenMsg, CwPacketAckMsg, CwPacketReceiveMsg,
    CwPacketTimeoutMsg,
};


use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{IbcPacket, Reply};
use cw_xcall_lib::network_address::NetId;

#[cw_serde]
pub enum ExecuteMsg {
    SetAdmin {
        address: String,
    },
    SetXCallHost {
        address: String,
    },
    SendMessage {
        to: NetId,
        sn: i64,
        msg: Vec<u8>,
    },
    ConfigureConnection {
        connection_id: String,
        counterparty_port_id: String,
        counterparty_nid: NetId,
        client_id: String,
        timeout_height: u64,
    },
    OverrideConnection {
        connection_id: String,
        counterparty_port_id: String,
        counterparty_nid: NetId,
        client_id: String,
        timeout_height: u64,
    },
    ClaimFees {
        nid: NetId,
        address: String,
    },

    SetFees {
        nid: NetId,
        packet_fee: u128,
        ack_fee: u128,
    },

    #[cfg(not(feature = "native_ibc"))]
    IbcChannelOpen {
        msg: CwChannelOpenMsg,
    },

    #[cfg(not(feature = "native_ibc"))]
    IbcChannelConnect {
        msg: CwChannelConnectMsg,
    },
    #[cfg(not(feature = "native_ibc"))]
    IbcChannelClose {
        msg: CwChannelCloseMsg,
    },
    #[cfg(not(feature = "native_ibc"))]
    IbcPacketReceive {
        msg: CwPacketReceiveMsg,
    },
    #[cfg(not(feature = "native_ibc"))]
    IbcPacketAck {
        msg: CwPacketAckMsg,
    },
    #[cfg(not(feature = "native_ibc"))]
    IbcPacketTimeout {
        msg: CwPacketTimeoutMsg,
    },
}

#[cw_serde]
pub struct ConfigResponse {
    pub channel_id: String,
    pub port: String,
    pub destination_channel_id: String,
    pub destination_port_id: String,
    pub light_client_id: String,
    pub timeout_height: u64,
}

#[cw_serde]
pub struct ChannelConfig {
    pub client_id: String,
    pub timeout_height: u64,
    pub counterparty_nid: NetId,
}


#[cw_serde]
pub struct ConnectionConfig {
    pub client_id: String,
    pub timeout_height: u64,
}

pub type IncomingPackets = Vec<((String, i64), IbcPacket)>;
pub type ConfiguredNetworks = Vec<((String, String), NetId)>;


#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetAdmin {},
    #[returns(u64)]
    GetTimeoutHeight { channel_id: String },
    #[returns(u64)]
    GetFee { nid: NetId, response: bool },
    #[returns(u64)]
    GetUnclaimedFee { nid: NetId, relayer: String },
    #[returns(ConfigResponse)]
    GetIbcConfig { nid: NetId },
    #[returns(ChannelConfig)]
    GetChannelConfig {
        channel_id: String,
    },
    #[returns(ConnectionConfig)]
    GetConnectionConfig {
        connection_id: String,
    },
    #[returns(IncomingPackets)]
    GetIncomingPackets {},

    #[returns(ConfiguredNetworks)]
    GetConfiguredNetworks {},

    #[returns(Vec<Reply>)]
    GetReplies {},

    #[returns(Vec<u8>)]
    GetIbcAcks {},

    #[returns(Vec<u8>)]
    GetIbcResponses {},
}
