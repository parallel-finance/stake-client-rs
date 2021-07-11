use codec::Encode;
use core::marker::PhantomData;
use frame_support::weights::Weight;
use frame_system::AccountInfo;
use substrate_subxt::balances::Balances;
use substrate_subxt::system::System as SubxtSystem;
use xcm::v0::{MultiAsset, MultiLocation};

#[module]
pub trait XcmPallet: Balances + SubxtSystem {}

// pub fn reserve_transfer_assets(
//     origin: OriginFor<T>,
//     dest: MultiLocation,
//     beneficiary: MultiLocation,
//     assets: Vec<MultiAsset>,
//     dest_weight: Weight,
#[derive(Call, Encode, Debug, Clone)]
pub struct ReserveTransferAssetsCall<T: XcmPallet> {
    pub dest: MultiLocation,
    pub beneficiary: MultiLocation,
    pub assets: Vec<MultiAsset>,
    pub dest_weight: Weight,
    pub _runtime: PhantomData<T>,
}

pub fn reserve_transfer_assets_call<T: XcmPallet>(
    dest: MultiLocation,
    beneficiary: MultiLocation,
    assets: Vec<MultiAsset>,
    dest_weight: Weight,
) -> ReserveTransferAssetsCall<T> {
    ReserveTransferAssetsCall::<T> {
        dest,
        beneficiary,
        assets,
        dest_weight,
        _runtime: PhantomData,
    }
}
