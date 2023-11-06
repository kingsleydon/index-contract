use crate::account::AccountType;
use crate::call::{Call, CallBuilder, CallParams, SubCall, SubExtrinsic};
use crate::step::Step;
use crate::utils::ToArray;
use scale::{Decode, Encode};
use xcm::{
    v2::{prelude::*, AssetId, Fungibility, Junctions, MultiAsset, MultiAssets, MultiLocation},
    VersionedMultiAssets, VersionedMultiLocation,
};

#[derive(Clone)]
pub struct PolkadotXcm {
    dest_chain_id: u32,
    account_type: AccountType,
}

impl PolkadotXcm {
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

impl CallBuilder for PolkadotXcm {
    fn build_call(&self, step: Step) -> Result<Call, &'static str> {
        let recipient = step.recipient;
        let asset_location: MultiLocation =
            Decode::decode(&mut step.spend_asset.as_slice()).map_err(|_| "InvalidMultilocation")?;
        let dest = VersionedMultiLocation::V2(MultiLocation::new(
            0,
            Junctions::X1(Parachain(self.dest_chain_id)),
        ));
        let beneficiary = VersionedMultiLocation::V2(MultiLocation::new(
            0,
            Junctions::X1(match &self.account_type {
                AccountType::Account20 => {
                    let recipient: [u8; 20] = recipient.to_array();
                    AccountKey20 {
                        network: NetworkId::Any,
                        key: recipient,
                    }
                }
                AccountType::Account32 => {
                    let recipient: [u8; 32] = recipient.to_array();
                    AccountId32 {
                        network: NetworkId::Any,
                        id: recipient,
                    }
                }
            }),
        ));
        let assets = VersionedMultiAssets::V2(MultiAssets::from(vec![MultiAsset {
            id: AssetId::Concrete(asset_location),
            fun: Fungibility::Fungible(step.spend_amount.ok_or("MissingSpendAmount")?),
        }]));

        let fee_asset_item: u32 = 0;

        Ok(Call {
            params: CallParams::Sub(SubCall {
                calldata: SubExtrinsic {
                    pallet_id: 0x63u8,
                    call_id: 0x02u8,
                    call: (dest, beneficiary, assets, fee_asset_item),
                }
                .encode(),
            }),
            input_call: None,
            call_index: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::PHALA_PARACHAIN_ID;

    #[test]
    fn test_bridge_to_phala() {
        let xcm = PolkadotXcm {
            dest_chain_id: PHALA_PARACHAIN_ID,
            account_type: AccountType::Account20,
        };
        let call = xcm
            .build_call(Step {
                exe: String::from(""),
                source_chain: String::from("Polkadot"),
                dest_chain: String::from("Phala"),
                spend_asset: hex::decode("0000").unwrap(),
                receive_asset: hex::decode("0100").unwrap(),
                sender: None,
                recipient: hex::decode(
                    "04dba0677fc274ffaccc0fa1030a66b171d1da9226d2bb9d152654e6a746f276",
                )
                .unwrap(),
                // 2 PHA
                spend_amount: Some(2_000_000_000_000 as u128),
                origin_balance: None,
                nonce: None,
            })
            .unwrap();

        match &call.params {
            CallParams::Sub(sub_call) => {
                println!("calldata: {:?}", hex::encode(&sub_call.calldata))
            }
            _ => assert!(false),
        }
    }
}
