use fuel_core_client::client::types::{block::Header, TransactionResponse};
use fuel_core_client::client::FuelClient;

use fuel_core_types::{fuel_tx::Receipt, fuel_types::Bytes32};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;
use tracing::{error, info, trace};

pub type BlockBody = (Bytes32, Option<TransactionResponse>, Option<Vec<Receipt>>);
pub type BlockBodies = Vec<BlockBody>;
pub type Blocks = Vec<(Header, BlockBodies)>;
pub type FetchBlockResult = Result<(Header, BlockBodies), BlockReaderError>;

pub struct BlockReader {
    batch_fetch_size: u64,
    client: FuelClient,
    block_handler: flume::Sender<Blocks>,
}

impl Drop for BlockReader {
    fn drop(&mut self) {
        info!("Block Rpc Reader drop");
    }
}

#[derive(Error, Debug)]
pub enum BlockReaderError {
    #[error("The latest height block: {0}")]
    HeightBlock(u64),
    #[error("Read block info from rpc failed: {0}")]
    ReadFromRpc(String),
    #[error("Sender failed the Handler channel maybe closed: {0}")]
    SendToHandler(String),
}

impl BlockReader {
    pub fn new(
        batch_fetch_size: u64,
        client: FuelClient,
        block_handler: flume::Sender<Blocks>,
    ) -> Self {
        Self {
            batch_fetch_size,
            client,
            block_handler,
        }
    }

    pub async fn start(&mut self, mut height: u64) -> Result<(), BlockReaderError> {
        loop {
            let fetch_feat = (height..(height + self.batch_fetch_size))
                .map(|h| Self::fetch_block(&self.client, h))
                .collect::<Vec<_>>();

            let maybe_blocks = futures::future::join_all(fetch_feat).await;
            let blocks = maybe_blocks
                .into_par_iter()
                .filter_map(|block| match block {
                    Ok(block) => Some(block),
                    Err(_) => None,
                })
                .collect::<Vec<_>>();

            if blocks.len() == 0 {
                info!("No blocks fetched, maybe the rpc is down");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            } else if (blocks.len() as u64) < self.batch_fetch_size {
                info!("Adjust batch fetch size to {}", blocks.len());
                self.batch_fetch_size = blocks.len() as u64;
            }

            height += blocks.len() as u64;
            self.block_handler
                .send(blocks)
                .map_err(|e| BlockReaderError::SendToHandler(e.to_string()))?;

            info!("Indexer Height {} wait for 100 millis", height);
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn fetch_block(client: &FuelClient, height: u64) -> FetchBlockResult {
        let block = match client
            .block_by_height(height)
            .await
            .map_err(|e| BlockReaderError::ReadFromRpc(e.to_string()))?
        {
            Some(block) => block,
            None => {
                trace!("no block at height {}", height);
                return Err(BlockReaderError::HeightBlock(height));
            }
        };

        let header = block.header;

        trace!(
            "block at height {} has {} txs",
            height,
            block.transactions.len()
        );

        let txs = block
            .transactions
            .iter()
            .map(|tx_hash| async move {
                let feat = client
                    .transaction(&tx_hash)
                    .await
                    .map_err(|e| BlockReaderError::ReadFromRpc(e.to_string()));
                let reseipts = client
                    .receipts(&tx_hash)
                    .await
                    .map_err(|e| BlockReaderError::ReadFromRpc(e.to_string()));
                (feat, reseipts, tx_hash)
            })
            .collect::<Vec<_>>();
        let mut transactions = vec![];

        let maybe_empty_txs = futures::future::join_all(txs).await;
        for (tx, reseipts, hash) in maybe_empty_txs {
            transactions.push((hash.clone(), tx?, reseipts?));
        }

        Ok((header, transactions))
    }
}
