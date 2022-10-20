mod abi;
mod pb;

use pb::ens;

use ethabi::ethereum_types::H256;
use substreams::prelude::*;
use sha3::{Digest, Keccak256};
use substreams::log;

use substreams_ethereum::pb::eth::v2::Block;

substreams_ethereum::init!();

// namehash('addr.reverse')
const ADDR_REVERSE_NODE: [u8; 32] =
    hex_literal::hex!("91d1777781884d03a6757a803996e38de2a42967fb37eeaca72729271025a9e2");

const ENS_REGISTRY_ADDRESS: [u8; 20] = hex_literal::hex!(
        "00000000000C2E074eC69A0dFb2997BA6C7d2e1e"
    );

#[substreams::handlers::map]
fn block_to_owner_mappings(blk: Block) -> Result<ens::OwnerMappings, substreams::errors::Error> {
    Ok(ens::OwnerMappings {
        mappings: blk.events::<abi::ENSRegistryWithFallback::events::NewOwner>(&[&ENS_REGISTRY_ADDRESS])
            .filter_map(|(new_owner_event, log)| {
                if new_owner_event.node != ADDR_REVERSE_NODE {
                    return None;
                }

                let calls = &log.receipt.transaction.calls;

                let (_ignored, call_index) = calls
                    .iter()
                    .flat_map(|call| call.logs.iter().map(move |log| (log, call.index)))
                    .find(|(inner_log, call)| inner_log.block_index == log.block_index())?;

                let call = &calls[call_index as usize];
                let parent_call = &calls[call.parent_index as usize];

                let found_element = [
                    &log.receipt.transaction.from,
                    &call.caller,
                    &parent_call.caller,
                ].into_iter().find(|address| {
                    let encoded_address = hex::encode(address);
                    let label_hash = Keccak256::digest(encoded_address).as_slice().to_vec();

                    return label_hash == new_owner_event.label
                })?;

                Some(ens::OwnerMapping {
                    label: new_owner_event.label.to_vec(),
                    owner: hex::encode(found_element),
                })
            }).collect()
    })
}
