mod abi;
mod pb;

use pb::ens;

use ethabi::ethereum_types::H256;
use sha3::{Digest, Keccak256};
use substreams::prelude::*;
use substreams::{log, proto, store};
use substreams::store::{StoreAddInt64, StoreGetProto, StoreSetProto};

use substreams_ethereum::pb::eth::v2::Block;

substreams_ethereum::init!();

// namehash('addr.reverse')
const ADDR_REVERSE_NODE: [u8; 32] =
    hex_literal::hex!("91d1777781884d03a6757a803996e38de2a42967fb37eeaca72729271025a9e2");

const ENS_REGISTRY_ADDRESS: [u8; 20] =
    hex_literal::hex!("00000000000C2E074eC69A0dFb2997BA6C7d2e1e");

#[substreams::handlers::map]
fn block_to_owner_mappings(blk: Block) -> Result<ens::OwnerMappings, substreams::errors::Error> {
    Ok(ens::OwnerMappings {
        mappings: blk
            .events::<abi::ENSRegistryWithFallback::events::NewOwner>(&[&ENS_REGISTRY_ADDRESS])
            .filter_map(|(new_owner_event, log)| {
                if new_owner_event.node != ADDR_REVERSE_NODE {
                    return None;
                }

                // let calls = &log.receipt.transaction.calls;

                // let (_ignored, call_index) = calls
                //     .iter()
                //     .flat_map(|call| call.logs.iter().map(move |log| (log, call.index)))
                //     .find(|(inner_log, call)| inner_log.block_index == log.block_index())?;

                // let call = &calls[call_index as usize];
                // let parent_call = &calls[call.parent_index as usize];

                let (address, label_hash) = [
                    &log.receipt.transaction.from,
                    // &call.caller,
                    // &parent_call.caller,
                ]
                .into_iter()
                .filter_map(|address| {
                    let encoded_address = hex::encode(address);
                    let label_hash = Keccak256::digest(encoded_address).as_slice().to_vec();

                    if label_hash == new_owner_event.label {
                        Some((address, label_hash))
                    } else {
                        None
                    }
                })
                .next()?;

                let mut hasher = Keccak256::new();
                hasher.update(ADDR_REVERSE_NODE);
                hasher.update(label_hash);
                let node = hasher.finalize();

                Some(ens::OwnerMapping {
                    node: hex::encode(node),
                    label: hex::encode(new_owner_event.label),
                    owner: hex::encode(address),
                    ordinal: log.ordinal(),
                })
            })
            .collect(),
    })
}

#[substreams::handlers::store]
fn owner_mappings_to_store(owners: ens::OwnerMappings, store: StoreSetProto<ens::OwnerMapping>) {
    for mapping in owners.mappings {
        store.set(mapping.ordinal, mapping.node.as_str(), &mapping)
    }
}

#[substreams::handlers::map]
fn block_to_resolver_mappings(
    blk: Block,
    owners: StoreGetProto<ens::OwnerMapping>,
) -> Result<ens::ResolverMappings, substreams::errors::Error> {
    Ok(ens::ResolverMappings {
        mappings: blk
            .events::<abi::ENSRegistryWithFallback::events::NewResolver>(&[&ENS_REGISTRY_ADDRESS])
            .filter_map(|(event, log)| {
                let mapping = owners.get_last(hex::encode(event.node))?;

                Some(ens::ResolverMapping {
                    node: mapping.node,
                    owner: mapping.owner,
                    resolver: hex::encode(event.resolver),
                    ordinal: log.ordinal(),
                })
            })
            .collect(),
    })
}

#[substreams::handlers::store]
fn aggregate(
    resolvers: ens::ResolverMappings,
    store: StoreAddInt64,
) {
    for mapping in resolvers.mappings {
        store.add(mapping.ordinal, mapping.resolver, 1);
    }
}