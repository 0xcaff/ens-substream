mod abi;
mod pb;

use pb::ens;

use ethabi::ethereum_types::H256;
use hex_literal::hex;
use substreams::prelude::*;
use sha3::{Digest, Keccak256};
use substreams::log;

use substreams_ethereum::pb::eth::v2::Block;

substreams_ethereum::init!();

// namehash('addr.reverse')
const ADDR_REVERSE_NODE: [u8; 32] =
    hex!("91d1777781884d03a6757a803996e38de2a42967fb37eeaca72729271025a9e2");

const ENS_REGISTRY_ADDRESS: [u8; 20] = hex!(
        "00000000000C2E074eC69A0dFb2997BA6C7d2e1e"
    );

#[substreams::handlers::map]
fn map_transfers(blk: Block) -> Result<ens::OwnerMappings, substreams::errors::Error> {
    Ok(ens::OwnerMappings {
        mappings: blk.events::<abi::ENSRegistryWithFallback::events::NewOwner>(&[&ENS_REGISTRY_ADDRESS])
            .filter_map(|(newOwnerEvent, log)| {
                if newOwnerEvent.node != ADDR_REVERSE_NODE {
                    return None;
                }

                let (_ignored, caller) = log
                    .receipt
                    .transaction
                    .calls
                    .iter()
                    .flat_map(|call| call.logs.iter().map(move |log| (log, call.caller.clone())))
                    .find(|(inner_log, call)| inner_log.block_index == log.block_index())?;

                let label_hash = Keccak256::digest(&caller);
                if label_hash.as_slice() != &newOwnerEvent.label {
                    log::info!("failed to match label_hash to owner");
                    return None;
                }

                Some(ens::OwnerMapping {
                    label: label_hash.as_slice().to_vec(),
                    owner: caller.clone(),
                })
            }).collect()
    })
}
