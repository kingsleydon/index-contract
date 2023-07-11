use alloc::{format, string::String, vec, vec::Vec};
use pink_extension::http_req;
use scale::Decode;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_number: u64,
    pub id: String,
    pub nonce: u64,
    pub result: bool,
    // unix timestamp
    pub timestamp: String,
    pub account: Vec<u8>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[serde(rename_all = "camelCase")]
struct Tx {
    pub id: String,
    pub account: String,
    pub nonce: u64,
    pub result: bool,
    pub block_number: u64,
    pub timestamp: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    transactions: Vec<Tx>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[serde(rename_all = "camelCase")]
struct ResponseData {
    data: QueryResult,
}

fn send_request(indexer: &str, query: &str) -> core::result::Result<Vec<u8>, &'static str> {
    let content_length = format!("{}", query.len());
    let headers: Vec<(String, String)> = vec![
        ("Content-Type".into(), "application/json".into()),
        ("Content-Length".into(), content_length),
    ];
    let response = http_req!("POST", indexer, query.into(), headers);

    if response.status_code != 200 {
        return Err("CallIndexerFailed");
    }

    Ok(response.body)
}

fn get_tx(
    indexer: &str,
    account: &[u8],
    nonce: u64,
) -> core::result::Result<Option<Transaction>, &'static str> {
    let account = format!("0x{}", hex::encode(account)).to_lowercase();
    pink_extension::debug!("get_tx: account: {}, nonce: {}", account, nonce);
    let query = format!(
        r#"{{ 
            "query": "query Query {{ transactions(where: {{nonce_eq: {nonce}, account_eq: \"{account}\" }}) {{ blockNumber id nonce result timestamp account }} }}",
            "variables": null,
            "operationName": "Query"
        }}"#
    );
    let body = send_request(indexer, &query)?;
    let response: ResponseData = pink_json::from_slice(&body).or(Err("InvalidBody"))?;
    let transactions = &response.data.transactions;

    pink_extension::debug!("get_tx: got transaction: {:?}", transactions);

    if transactions.len() != 1 {
        return Ok(None);
    }

    let tx = &response.data.transactions[0];

    Ok(Some(Transaction {
        block_number: tx.block_number,
        id: tx.id.clone(),
        nonce: tx.nonce,
        result: tx.result,
        timestamp: tx.timestamp.clone(),
        account: hex::decode(&tx.account[2..]).or(Err("InvalidAddress"))?,
    }))
}

/// Return true if transaction is confirmed on chain
pub fn check_tx(indexer: &str, account: &[u8], nonce: u64) -> Result<bool, &'static str> {
    // nonce from storage is one larger than the last tx's nonce
    let tx = get_tx(indexer, account, nonce)?;
    pink_extension::debug!(
        "check_tx: tx record returned from off-chain indexer: {:?}",
        tx
    );
    if let Some(tx) = tx {
        return Ok(tx.result);
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore]
    fn should_work() {
        pink_extension_runtime::mock_ext::mock_all_ext();
        let account = hex_literal::hex!("9ccbdac25ecda4d817b3aa0e020bc65f841c80c3");
        let tx = get_tx("http://127.0.0.1:4350", &account, 1)
            .unwrap()
            .unwrap();
        dbg!(&tx);
        assert_eq!(tx.result, true);
    }
}
