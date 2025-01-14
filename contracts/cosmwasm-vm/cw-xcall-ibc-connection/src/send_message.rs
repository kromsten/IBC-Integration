use common::rlp::{self, Nullable};
use cosmwasm_std::{to_binary, Coin, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response, Storage, SubMsg, Uint128};
use cw_xcall_lib::network_address::NetId;

use crate::{
    error::ContractError,
    state::{CwIbcConnection, IbcConfig, HOST_SEND_MESSAGE_REPLY_ID},
    types::{message::Message, LOG_PREFIX},
};

pub const HOST_FORWARD_REPLY_ID : u64 = 1;

impl<'a> CwIbcConnection<'a> {
    pub fn send_message(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        nid: NetId,
        sn: i64,
        message: Vec<u8>,
    ) -> Result<Response, ContractError> {
        self.ensure_xcall_handler(deps.as_ref().storage, info.sender)?;

        println!("{LOG_PREFIX} Packet Validated");
        let ibc_config = self.get_ibc_config(deps.as_ref().storage, &nid)?;

        if sn < 0 {
            return self.write_acknowledgement(deps.storage, &ibc_config, message, -sn);
        }

        let network_fee = self.get_network_fees(deps.as_ref().storage, nid.clone());
        let mut total_fee = network_fee.send_packet_fee;


        let config = self.get_config(deps.storage)?;
        let fund = get_amount_for_denom(&info.funds, config.denom);

        if fund < total_fee.into() {
            return Err(ContractError::InsufficientFunds {});
        }


        let msg = Message {
            sn: Nullable::new(Some(sn)),
            fee: network_fee.send_packet_fee,
            data: message,
        };

        #[cfg(feature = "native_ibc")]
        {
            let packet = self.create_request_packet(env, ibc_config, msg.clone())?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_always(packet, HOST_SEND_MESSAGE_REPLY_ID))
                .add_attribute("method", "send_message"))
        }

        #[cfg(not(feature = "native_ibc"))]
        {

            let sequence_number_host = self.query_host_sequence_no(deps.as_ref(), &ibc_config)?;
            
            if sn > 0 {
                total_fee += network_fee.ack_fee;
                self.add_unclaimed_ack_fees(
                    deps.storage,
                    &nid,
                    sequence_number_host,
                    network_fee.ack_fee,
                )?;
            }

            let timeout_height = self.query_timeout_height(deps.as_ref(), &ibc_config.src_endpoint().channel_id)?;

            let packet_data =
                self.create_packet(ibc_config, timeout_height, sequence_number_host, msg);

            println!("{} Raw Packet Created {:?}", LOG_PREFIX, &packet_data);

            let submessage = self.call_host_send_message(deps, packet_data)?;
            Ok(Response::new()
                .add_submessage(submessage)
                .add_attribute("method", "send_message"))
        }
    }

    fn write_acknowledgement(
        &self,
        store: &mut dyn Storage,
        config: &IbcConfig,
        msg: Vec<u8>,
        sn: i64,
    ) -> Result<Response, ContractError> {
        let channel_id = config.src_endpoint().channel_id.clone();
        let packet = self.get_incoming_packet(store, &channel_id, sn)?;
        self.remove_incoming_packet(store, &channel_id, sn);
        let submsg = self.call_host_write_acknowledgement(store, packet, msg)?;
        Ok(Response::new().add_submessage(submsg))
    }
}

fn get_amount_for_denom(funds: &Vec<Coin>, target_denom: String) -> Uint128 {
    for coin in funds.iter() {
        if coin.denom == target_denom {
            return coin.amount;
        }
    }
    Uint128::zero()
}
#[cfg(feature = "native_ibc")]
impl<'a> CwIbcConnection<'a> {
    /// This function creates an IBC message to send a packet with a timeout to a destination endpoint.
    ///
    /// Arguments:
    ///
    /// * `deps`: `deps` is a mutable reference to the dependencies of the contract. It is used to
    /// interact with the storage and other modules of the contract.
    /// * `env`: `env` is an object that contains information about the current blockchain environment,
    /// such as the current block height, time, and chain ID. It is used to calculate the timeout for the
    /// IBC packet.
    /// * `time_out_height`: The height of the block at which the timeout for the packet will occur.
    /// * `message`: `message` is a `CallServiceMessage` struct that contains the information needed to
    /// create a request packet to be sent over the IBC channel. This includes the method name, input
    /// arguments, and any other relevant data needed for the service call.
    ///
    /// Returns:
    ///
    /// a `Result` with an `IbcMsg` on success or a `ContractError` on failure.
    fn create_request_packet(
        &self,
        env: Env,
        ibc_config: IbcConfig,
        message: Message,
    ) -> Result<IbcMsg, ContractError> {

        let timeout = IbcTimeout::with_timestamp(env.block.time.plus_seconds(300));
        let encoded = rlp::encode(&message).to_vec();

        Ok(IbcMsg::SendPacket {
            channel_id: ibc_config.src_endpoint().channel_id.clone(),
            data: to_binary(&encoded)?,
            timeout,
        })
    }
}
