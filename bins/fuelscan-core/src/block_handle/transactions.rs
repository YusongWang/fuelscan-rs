use anyhow::Result;
use fuel_core_client::client::{schema::block::Header, types::TransactionResponse};
use models::transaction::Transaction;

pub fn init_transaction_with_txs() -> Vec<Transaction> {
    vec![]
}

pub fn insert_tx(header: &Header, (hash, tx): &(String, TransactionResponse)) -> Result<()> {
    dbg!("insert transactions", header);
    /*         let mut input = PutItemInput {
        table_name: "transactions".to_string(),
        ..Default::default()
    };

    let mut item: HashMap<String, AttributeValue> = HashMap::new();
    let table_type = AttributeValue {
        s: Some("transactions".to_owned()),
        ..Default::default()
    };
    item.insert("table_type".into(), table_type);

    let hash: AttributeValue = AttributeValue {
        s: Some(hash.to_string()),
        ..Default::default()
    };
    item.insert("hash".into(), hash);

    let hash = header.id.to_string();

    let block_hash: AttributeValue = AttributeValue {
        s: Some(header.id.to_string()),
        ..Default::default()
    };
    item.insert("block_hash".into(), block_hash);

    let height = AttributeValue {
        n: Some(format!("{}", header.height.0)),
        ..Default::default()
    };
    item.insert("height".into(), height);

    let da_height = AttributeValue {
        n: Some(format!("{}", header.da_height.0)),
        ..Default::default()
    };
    item.insert("da_height".into(), da_height);

    let status = serde_json::to_string(&tx.status).map_err(|e| {
        BlockHandlerError::InsertDb(format!("failed to serialize status: {}", e))
    })?;

    let status: AttributeValue = AttributeValue {
        s: Some(status),
        ..Default::default()
    };
    item.insert("status".into(), status);

    if tx.transaction.is_create() {
        //contract deplyment or somethings
        let create = tx.transaction.as_create().unwrap();
        let transaction_type = AttributeValue {
            s: Some("create".into()),
            ..Default::default()
        };
        item.insert("transaction_type".into(), transaction_type);

        let gas_price = AttributeValue {
            n: Some(format!("{}", create.gas_price())),
            ..Default::default()
        };
        item.insert("gas_price".into(), gas_price);
        let gas_limit = AttributeValue {
            n: Some(format!("{}", create.gas_limit())),
            ..Default::default()
        };
        item.insert("gas_limit".into(), gas_limit);

        let mut contract_address = "".to_string();

        for input in create.inputs() {
            if let fuel_core_types::fuel_tx::Input::CoinSigned {
                utxo_id: _,
                owner,
                amount: _,
                asset_id: _,
                tx_pointer: _,
                witness_index: _,
                maturity: _,
            } = input
            {
                let sender: AttributeValue = AttributeValue {
                    s: Some(owner.clone().to_string()),
                    ..Default::default()
                };
                item.insert("sender".into(), sender);
            }

            if let fuel_core_types::fuel_tx::Input::Contract {
                utxo_id: _,
                balance_root: _,
                state_root: _,
                tx_pointer: _,
                contract_id,
            } = input
            {
                contract_address = contract_id.clone().to_string();

                let contract: AttributeValue = AttributeValue {
                    s: Some(contract_id.clone().to_string()),
                    ..Default::default()
                };
                item.insert("contract_address".into(), contract);
            }
        }

        let input = serde_json::to_string(create.inputs())
            .map_err(|e| BlockHandlerError::SerdeJson(e.to_string()))?;
        let input: AttributeValue = AttributeValue {
            s: Some(input),
            ..Default::default()
        };
        item.insert("input".into(), input);

        let output = serde_json::to_string(create.outputs())
            .map_err(|e| BlockHandlerError::SerdeJson(e.to_string()))?;
        let output: AttributeValue = AttributeValue {
            s: Some(output),
            ..Default::default()
        };
        item.insert("output".into(), output);

        let byte_code_indexer = create.bytecode_witness_index();
        let witnesses = create.witnesses().clone();
        let bytecode = witnesses[*byte_code_indexer as usize].as_vec().clone();

        self.db
            .set_raw(
                DatabaseName::Contract,
                &hex::decode(contract_address.trim_start_matches("0x")).unwrap(),
                &bytecode,
            )
            .await
            .map_err(|e| BlockHandlerError::InsertDb(e.to_string()))?;
    } else if tx.transaction.is_mint() {
        //system Msg?
        let mint = tx.transaction.as_mint().unwrap();
        mint.outputs();
        let transaction_type = AttributeValue {
            s: Some("mint".into()),
            ..Default::default()
        };
        item.insert("transaction_type".into(), transaction_type);
        for output in mint.outputs() {
            if let fuel_core_types::fuel_tx::Output::Coin {
                to: owner,
                amount: _,
                asset_id: _,
            } = output
            {
                let receiver: AttributeValue = AttributeValue {
                    s: Some(owner.clone().to_string()),
                    ..Default::default()
                };
                item.insert("receiver".into(), receiver);
            }
        }

        let output = serde_json::to_string(mint.outputs())
            .map_err(|e| BlockHandlerError::SerdeJson(e.to_string()))?;
        let output: AttributeValue = AttributeValue {
            s: Some(output),
            ..Default::default()
        };
        item.insert("output".into(), output);
    } else if tx.transaction.is_script() {
        //transfer or contract call
        let script = tx.transaction.as_script().unwrap();

        let transaction_type = AttributeValue {
            s: Some("script".into()),
            ..Default::default()
        };
        item.insert("transaction_type".into(), transaction_type);

        let gas_price = AttributeValue {
            n: Some(format!("{}", script.gas_price())),
            ..Default::default()
        };
        item.insert("gas_price".into(), gas_price);
        let gas_limit = AttributeValue {
            n: Some(format!("{}", script.gas_limit())),
            ..Default::default()
        };
        item.insert("gas_limit".into(), gas_limit);

        for input in script.inputs() {
            if let fuel_core_types::fuel_tx::Input::CoinSigned {
                utxo_id: _,
                owner,
                amount: _,
                asset_id: _,
                tx_pointer: _,
                witness_index: _,
                maturity: _,
            } = input
            {
                let sender: AttributeValue = AttributeValue {
                    s: Some(owner.clone().to_string()),
                    ..Default::default()
                };
                item.insert("sender".into(), sender);
            }
        }

        let input = serde_json::to_string(script.inputs())
            .map_err(|e| BlockHandlerError::SerdeJson(e.to_string()))?;
        let input: AttributeValue = AttributeValue {
            s: Some(input),
            ..Default::default()
        };
        item.insert("input".into(), input);

        let output = serde_json::to_string(script.outputs())
            .map_err(|e| BlockHandlerError::SerdeJson(e.to_string()))?;
        let output: AttributeValue = AttributeValue {
            s: Some(output),
            ..Default::default()
        };

        item.insert("output".into(), output);
        let bytecode = script.script_data().clone();
        if !bytecode.is_empty() {
            self.db
                .set_raw(
                    DatabaseName::ScriptData,
                    &hex::decode(hash.trim_start_matches("0x")).unwrap(),
                    &bytecode,
                )
                .await
                .map_err(|e| BlockHandlerError::InsertDb(e.to_string()))?;
        }

        let bytecode = script.script().clone();
        if !bytecode.is_empty() {
            self.db
                .set_raw(
                    DatabaseName::Script,
                    &hex::decode(hash.trim_start_matches("0x")).unwrap(),
                    &bytecode,
                )
                .await
                .map_err(|e| BlockHandlerError::InsertDb(e.to_string()))?;
        }
    } else {
        unimplemented!();
    }

    input.item = item;
    let _ = self
        .db_client
        .put_item(input)
        .await
        .map_err(|e| BlockHandlerError::InsertTransactionDb(e.to_string()))?; */

    Ok(())
}
