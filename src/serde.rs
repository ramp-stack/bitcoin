use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use bdk_core::{BlockId, CheckPoint, TxUpdate};
use bdk_core::spk_client::{FullScanRequest, FullScanResponse};

use bdk_wallet::bitcoin::{BlockHash, OutPoint, ScriptBuf, Transaction, TxOut, Txid, Network};

#[derive(Serialize, Deserialize)]
struct SerdeTxUpdate<A: Ord> {
    txs: Vec<Transaction>,
    txouts: BTreeMap<OutPoint, TxOut>,
    anchors: BTreeSet<(A, Txid)>,
    seen_ats: HashSet<(Txid, u64)>,
    evicted_ats: HashSet<(Txid, u64)>,
}

impl<A: Ord> From<TxUpdate<A>> for SerdeTxUpdate<A> {
    fn from(tx_update: TxUpdate<A>) -> SerdeTxUpdate<A> {
        SerdeTxUpdate {
            txs: tx_update.txs.into_iter().map(|arc_tx| (*arc_tx).clone()).collect(),
            txouts: tx_update.txouts,
            anchors: tx_update.anchors,
            seen_ats: tx_update.seen_ats,
            evicted_ats: tx_update.evicted_ats
        }
    }
}

impl<A: Ord> From<SerdeTxUpdate<A>> for TxUpdate<A> {
    fn from(val: SerdeTxUpdate<A>) -> Self {
        let mut tx_update = TxUpdate::default();
        tx_update.txs = val.txs.into_iter().map(Arc::new).collect();
        tx_update.txouts = val.txouts;
        tx_update.anchors = val.anchors;
        tx_update.seen_ats = val.seen_ats;
        tx_update.evicted_ats = val.evicted_ats;
        tx_update
    }
}


#[derive(Serialize, Deserialize)]
pub struct SerdeBlockId {
    pub height: u32,
    pub hash: BlockHash,
}

impl From<BlockId> for SerdeBlockId {
    fn from(block_id: BlockId) -> SerdeBlockId {
        SerdeBlockId {
            height: block_id.height,
            hash: block_id.hash,
        }
    }
}

impl From<SerdeBlockId> for BlockId {
    fn from(val: SerdeBlockId) -> Self {
        BlockId {
            height: val.height,
            hash: val.hash
        }
    }
}


#[derive(Serialize, Deserialize)]
struct SerdeCheckPoint {
    block_ids: Vec<SerdeBlockId>
}

impl From<CheckPoint> for SerdeCheckPoint {
    fn from(cp: CheckPoint) -> SerdeCheckPoint {
        SerdeCheckPoint{
            block_ids: cp.into_iter().map(|cp| cp.block_id().into()).collect()
        }
    } 
}

impl From<SerdeCheckPoint> for CheckPoint {
    fn from(serde_cp: SerdeCheckPoint) -> CheckPoint {
        CheckPoint::from_block_ids(
            serde_cp.block_ids.into_iter().map(|id| id.into())
        ).expect("ERROR")
    }
}

#[derive(Serialize, Deserialize)]
struct SerdeFullScanResponse<K: Ord, A: Ord> {
    tx_update: SerdeTxUpdate<A>,
    last_active_indices: BTreeMap<K, u32>,
    chain_update: Option<SerdeCheckPoint>,
}


impl<K: Ord, A: Ord> From<FullScanResponse<K, A>> for SerdeFullScanResponse<K, A> {
    fn from(full_scan_response: FullScanResponse<K, A>) -> SerdeFullScanResponse<K, A> {
        SerdeFullScanResponse {
            tx_update: full_scan_response.tx_update.into(),
            last_active_indices: full_scan_response.last_active_indices,
            chain_update: full_scan_response.chain_update.map(SerdeCheckPoint::from),
        }
    }
}

impl<K: Ord, A: Ord> From<SerdeFullScanResponse<K, A>> for FullScanResponse<K, A> {
    fn from(val: SerdeFullScanResponse<K, A>) -> Self {
        FullScanResponse {
            tx_update: val.tx_update.into(),
            last_active_indices: val.last_active_indices,
            chain_update: val.chain_update.map(|cu| cu.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerdeFullScanRequest<K: Ord + Clone> {
    start_time: u64,
    chain_tip: Option<SerdeCheckPoint>,
    keychains: Vec<K>,
    keychain_scripts: Vec<Vec<ScriptBuf>>,
}

impl<K: Ord + Clone> From<FullScanRequest<K>> for SerdeFullScanRequest<K> {
    fn from(mut full_scan_request: FullScanRequest<K>) -> SerdeFullScanRequest<K> {
        let keychains: Vec<K> = full_scan_request.keychains().into_iter().collect();
        let mut keychain_scripts: Vec<Vec<ScriptBuf>> = Vec::new();

        for keychain in &keychains {
            let scripts: Vec<ScriptBuf> = full_scan_request
                .iter_spks(keychain.clone())
                .map(|(_, script)| script)
                .collect();
            keychain_scripts.push(scripts);
        }

        SerdeFullScanRequest {
            start_time: full_scan_request.start_time(),
            chain_tip: full_scan_request.chain_tip().map(|cp| cp.into()),
            keychains,
            keychain_scripts,
        }
    }
}


impl<K: Ord + Clone> From<SerdeFullScanRequest<K>> for FullScanRequest<K> {
    fn from(val: SerdeFullScanRequest<K>) -> Self {
        let mut builder = FullScanRequest::builder_at(val.start_time);
        if let Some(chain_tip) = val.chain_tip {
            builder = builder.chain_tip(chain_tip.into());
        }

        for (keychain, scripts) in val.keychains.into_iter().zip(val.keychain_scripts.into_iter()) {
            let indexed_scripts = scripts.into_iter().enumerate().map(|(index, script)| {
                (index as u32, script)
            });
            builder = builder.spks_for_keychain(keychain, indexed_scripts);
        }

        builder.build()
    }
}

