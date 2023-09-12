use alloc::{vec, vec::Vec};
use scale::{Decode, Encode};

use crate::call::{Call, CallBuilder, CallParams, SubCall, SubExtrinsic};
use crate::step::Step;

use crate::utils::ToArray;
use xcm::v3::{prelude::*, AssetId, Fungibility, Junctions, MultiAsset, MultiLocation, Weight};

use crate::account::AccountType;

#[derive(Clone)]
pub struct XTransferXcm {
    dest_chain_id: u32,
    // dest chain account type
    account_type: AccountType,
}

impl XTransferXcm {
    pub fn new(dest_chain_id: u32, account_type: AccountType) -> Self
    where
        Self: Sized,
    {
        Self {
            dest_chain_id,
            account_type,
        }
    }
}

impl CallBuilder for XTransferXcm {
    fn build_call(&self, step: Step) -> Result<Vec<Call>, &'static str> {
        let recipient = step.recipient.ok_or("MissingRecipient")?;
        let asset_location: MultiLocation =
            Decode::decode(&mut step.spend_asset.as_slice()).map_err(|_| "InvalidMultilocation")?;
        let multi_asset = MultiAsset {
            id: AssetId::Concrete(asset_location),
            fun: Fungibility::Fungible(step.spend_amount.ok_or("MissingSpendAmount")?),
        };
        let dest = MultiLocation::new(
            1,
            Junctions::X2(
                Parachain(self.dest_chain_id),
                match &self.account_type {
                    AccountType::Account20 => {
                        let recipient: [u8; 20] = recipient.to_array();
                        AccountKey20 {
                            network: None,
                            key: recipient,
                        }
                    }
                    AccountType::Account32 => {
                        let recipient: [u8; 32] = recipient.to_array();
                        AccountId32 {
                            network: None,
                            id: recipient,
                        }
                    }
                },
            ),
        );
        let dest_weight: Weight = Weight::from_parts(6000000000_u64, 1000000_u64);

        Ok(vec![Call {
            params: CallParams::Sub(SubCall {
                calldata: SubExtrinsic {
                    pallet_id: 0x52u8,
                    call_id: 0x0u8,
                    call: (multi_asset, dest, Some(dest_weight)),
                }
                .encode(),
            }),
            input_call: None,
            call_index: None,
        }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_to_astar() {
        let xtransfer = XTransferXcm {
            dest_chain_id: 2006,
            // dest chain account type
            account_type: AccountType::Account32,
        };
        let calls = xtransfer
            .build_call(Step {
                exe: String::from(""),
                source_chain: String::from("Phala"),
                dest_chain: String::from("Astar"),
                spend_asset: hex::decode("0000").unwrap(),
                receive_asset: hex::decode("010100cd1f").unwrap(),
                sender: None,
                recipient: Some(
                    hex::decode("04dba0677fc274ffaccc0fa1030a66b171d1da9226d2bb9d152654e6a746f276")
                        .unwrap(),
                ),
                // 2 PHA
                spend_amount: Some(2_000_000_000_000 as u128),
                origin_balance: None,
                nonce: None,
            })
            .unwrap();

        match &calls[0].params {
            CallParams::Sub(sub_call) => {
                println!("calldata: {:?}", hex::encode(&sub_call.calldata))
            }
            _ => assert!(false),
        }
    }
}
