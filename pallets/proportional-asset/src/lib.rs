//! # Proportional Asset Pallet
//!
//! The Proportional Asset pallet provides the functionality for sharing the ownership of an asset.
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Proportional Asset Pallet itself provides the mechanism for sharing the owneship of an asset btween stakeholders,
//! offering portions of this asset, buying and transfering portions.
//!
//! By way of example, a House from a Real estate could be represented as a Proportional Asset.
//!
//!
//! ### Terminology
//!
//! - **Identifier:** A unique id (Hash) representing a specific asset.
//! - **Main owner:** An account who owns 50% or more from the asset.
//! - **Owner:** An account which holds any portion of the asset.
//! - **MetaData:** Data representing per owner representing the shares of the owner,
//! the available offers and the price that the owner has set.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! General spending/proposal protocol:
//! - `create_proportional_asset` - Create a proportional asset with 100% ownership for the caller.
//! - `offer_shares` - Allows an owner of a portion to make offers for an amount of shares.
//! - `buy_shares` - Allows accounts to buy offered shared for the specified price.
//! - `transfer_shares_to_account` - Transfers shares to an account (For free!)
//! - `claim_ownership` - Claims the main ownership of an asset.
//!
//! The Proportional Asset pallet is loosely coupled with Balances.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::{traits::Currency, PalletId};

use frame_support::{inherent::Vec, traits::ExistenceRequirement::AllowDeath};
use sp_runtime::traits::Hash;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const PALLET_ID: PalletId = PalletId(*b"Asset#*!");

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Metadata struct represents data for each proportional ownership.
	#[derive(
		Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo, MaxEncodedLen,
	)]
	pub struct MetaData {
		pub offers: u64,
		pub shares: u64,
		pub price: u64,
	}

	/// TOTAL_SUPPLY constant is the divisor of the asset (percentage).
	pub const TOTAL_SUPPLY: u64 = 100;

	/// Identifier is the Hash representing uniquely an asset.
	pub type Identifier<T> = <T as frame_system::Config>::Hash;

	/// ProportionalAssetToOwnerToMetadata is the MetaData that each owner has for an asset.
	#[pallet::storage]
	pub type ProportionalAssetToOwnerToMetadata<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		Identifier<T>,
		Blake2_128Concat,
		T::AccountId,
		MetaData,
	>;

	/// ProportionalAssetToMainOwner is the main owner of an asset
	#[pallet::storage]
	pub type ProportionalAssetToMainOwner<T: Config> =
		StorageMap<_, Blake2_128Concat, Identifier<T>, T::AccountId>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The event configured from the runtime
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The currency configured from the runtime
		type Currency: Currency<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new asset is initialized
		ProportionalAssetInitialized(Identifier<T>, T::AccountId),
		/// New shares have been offerred
		SharesOffered(Identifier<T>, u64),
		/// Shares have been transferrred
		SharesTransferred(T::AccountId, T::AccountId, u64),
		/// The main owner has changed
		MainOwnerSet(T::AccountId, Identifier<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The asset does not exist
		AssetDoesNotExist,
		/// The asset already exists
		AssetAlreadyExists,
		/// The offers are invalid
		InvalidOffers,
		/// The amount sent is incorrect
		IncorrectAmount,
		/// Cannot convert balances
		ConversionError,
		/// The shares are incorrect
		IncorrectSharesSelection,
		/// The seller selected is incorrect
		IncorrectSeller,
		/// The account is not the main owner of the asset
		NotMainOwner,
		/// The account is already the main owner of the asset
		AlreadyMainOnwer,
		/// The shares are not enough
		NotEnoughShares,
		/// The account is not valid
		InvalidAccount,
		/// The balance is not enough
		InsufficientBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new proportional asset
		///
		/// The Hash of the data passed should not result to an existing asset identifier.
		///
		/// A proportional asset gets created successfully and allocated to the main owner
		/// else the call fails
		///
		/// If the call is success, the metadata added for the origin
		/// are TOTAL_SUPPLY number of shares, 0 offers and the specified share_price
		///
		/// - `data`: The data information about the asset.
		/// - `share_price`: The share price for the origin's shares
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn create_proportional_asset(
			origin: OriginFor<T>,
			data: Vec<u8>,
			share_price: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// get a hash of the data
			let id = T::Hashing::hash(&data);

			// Check if id exists
			match ProportionalAssetToOwnerToMetadata::<T>::get(id, who.clone()) {
				Some(_metadata) => Err(Error::<T>::AssetAlreadyExists)?,
				None => {
					let metadata = MetaData { shares: TOTAL_SUPPLY, offers: 0, price: share_price };

					// Create the asset & set the owner
					// Initialize owner with all the supply
					// Initialize assets offers to 0
					ProportionalAssetToOwnerToMetadata::<T>::set(id, &who, Some(metadata));

					// Set the main owner of the asset
					ProportionalAssetToMainOwner::<T>::set(id, Some(who.clone()));

					Self::deposit_event(Event::ProportionalAssetInitialized(id, who));

					Ok(())
				},
			}
		}

		/// Offers new shares for sale
		///
		/// The origin should own at least the amount to be offerred.
		///
		/// The offers for the metadata of the origin is successfully updated
		/// else the call fails.
		///
		/// If the call is success, the metadata added for the origin
		/// are updated with the new price and the offers are incremented by
		/// the shares to be offerred
		///
		/// - `id`: The identifier of the asset
		/// - `shares_to_offer`: The amount of shares to be offerred
		/// - `share_price`: The price to offer each portion
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 1))]
		pub fn offer_shares(
			origin: OriginFor<T>,
			id: Identifier<T>,
			shares_to_offer: u64,
			share_price: u64,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(Self::is_owner_of(&who, &id), Error::<T>::NotMainOwner);

			match ProportionalAssetToOwnerToMetadata::<T>::get(id, who.clone()) {
				None => Err(Error::<T>::InvalidAccount)?,
				Some(metadata) => {
					ensure!(&shares_to_offer.le(&metadata.shares), Error::<T>::InvalidOffers);

					let new_metadata = MetaData {
						shares: metadata.shares,
						offers: shares_to_offer,
						price: share_price,
					};

					ProportionalAssetToOwnerToMetadata::<T>::set(id, who, Some(new_metadata));

					Self::deposit_event(Event::SharesOffered(id, share_price));

					Ok(())
				},
			}
		}

		/// Transfers shares for free
		///
		/// The origin should own the at least the amount to be transferred.
		///
		/// The offers for the metadata of the origin are successfully updated
		/// else the call fails.
		///
		/// If the call is success, the specified shares will belong to the recipient.
		///
		/// - `id`: The identifier of the asset
		/// - `amount`: The amount of shares to be transferred
		/// - `to`: The recipient account
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 1))]
		pub fn transfer_shares_to_account(
			origin: OriginFor<T>,
			id: Identifier<T>,
			amount: u64,
			to: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match ProportionalAssetToOwnerToMetadata::<T>::get(id, who.clone()) {
				None => Err(Error::<T>::InvalidAccount)?,
				Some(origin_metadata) => {
					ensure!(
						origin_metadata.shares.ge(&amount),
						Error::<T>::IncorrectSharesSelection
					);

					// Decrease origin shares
					let new_origin_shares = origin_metadata.shares.saturating_sub(amount);

					match ProportionalAssetToOwnerToMetadata::<T>::get(id, to.clone()) {
						None => {
							let new_metadata = MetaData { shares: amount, offers: 0, price: 0 };

							ProportionalAssetToOwnerToMetadata::<T>::set(
								id,
								to.clone(),
								Some(new_metadata),
							);
						},
						Some(metadata) => {
							// Increase to shares
							let new_to_shares = metadata.shares.saturating_add(amount);

							let new_metadata = MetaData {
								shares: new_to_shares,
								offers: metadata.offers, //TODO: Fix offers
								price: metadata.price,
							};

							ProportionalAssetToOwnerToMetadata::<T>::set(
								id,
								to.clone(),
								Some(new_metadata),
							);
						},
					}

					// Update the origin metadata
					let new_origin_metadata = MetaData {
						shares: new_origin_shares,
						offers: origin_metadata.offers,
						price: origin_metadata.price,
					};

					ProportionalAssetToOwnerToMetadata::<T>::set(
						id,
						who.clone(),
						Some(new_origin_metadata),
					);

					Self::deposit_event(Event::SharesTransferred(who, to, amount));

					Ok(())
				},
			}
		}

		/// Buy offerred shares
		///
		/// The origin should own have balance more than the amount sent
		/// and the seller should own at least the specified shares
		///
		/// The shares are transferred to the origin
		/// and the amount transferred to the seller
		///
		///
		/// - `id`: The identifier of the asset
		/// - `shares_to_buy`: The amount of shares to be be purchased
		/// - `amount`: The amount sent for payment
		/// - `from`: The seller
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(5, 3))]
		pub fn buy_shares(
			origin: OriginFor<T>,
			id: Identifier<T>,
			shares_to_buy: u64,
			amount: BalanceOf<T>,
			from: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// // Ensure that the sender is not the seller
			ensure!(who != from, Error::<T>::IncorrectSeller);

			// // Ensure that "from" is the owner of the asset
			ensure!(Self::is_owner_of(&from, &id), Error::<T>::IncorrectSeller);

			match ProportionalAssetToOwnerToMetadata::<T>::get(id, from.clone()) {
				None => Err(Error::<T>::InvalidAccount)?,
				Some(from_metadata) => {
					// make sure that "from" owns more than the specified shares_to_buy
					ensure!(from_metadata.shares.ge(&shares_to_buy), Error::<T>::IncorrectAmount);

					// Calculate the correct price
					let price = from_metadata.price.saturating_mul(shares_to_buy);

					let parsed_amount_sent =
						Self::balance_to_u64_option(amount).ok_or(Error::<T>::ConversionError)?;

					// Ensure that the amount sent is correnct
					ensure!(parsed_amount_sent.ge(&price), Error::<T>::IncorrectAmount);

					// Make sure that shares_to_buy <= from's shares
					ensure!(
						shares_to_buy.le(&from_metadata.shares),
						Error::<T>::IncorrectSharesSelection
					);

					// Make sure that shares_to_buy <= offered
					ensure!(
						shares_to_buy.le(&from_metadata.offers),
						Error::<T>::IncorrectSharesSelection
					);

					// Decrease for owner of the share
					let new_from_shares = from_metadata.shares.saturating_sub(shares_to_buy);

					// Update from offers
					let new_from_offers = from_metadata.offers.saturating_sub(shares_to_buy);

					let new_from_metadata = MetaData {
						shares: new_from_shares,
						offers: new_from_offers,
						price: from_metadata.price,
					};

					// Calculate new shares of origin
					// get origin shares, if it doesn't have any just set the new amount
					let mut new_origin_metadata = MetaData { shares: 0, offers: 0, price: 0 };

					match ProportionalAssetToOwnerToMetadata::<T>::get(id, who.clone()) {
						None => new_origin_metadata.shares = shares_to_buy,
						Some(old_origin_metadata) => {
							new_origin_metadata.shares =
								old_origin_metadata.shares.saturating_add(shares_to_buy);
						},
					}

					// Ensure that origin has the correct amount of Currency
					ensure!(
						T::Currency::free_balance(&who).ge(&amount),
						<Error<T>>::InsufficientBalance
					);

					T::Currency::transfer(&who, &from, amount, AllowDeath)
						.map_err(|_| DispatchError::Other("Can't transfer currency"))?;

					// Update storage

					ProportionalAssetToOwnerToMetadata::<T>::set(
						id,
						who.clone(),
						Some(new_origin_metadata),
					);

					ProportionalAssetToOwnerToMetadata::<T>::set(
						id,
						from.clone(),
						Some(new_from_metadata),
					);

					Self::deposit_event(Event::SharesTransferred(from, who, shares_to_buy));

					Ok(())
				},
			}
		}

		/// Claim main ownership of the asset
		///
		/// The origin should own more than 1/2 of the asset.
		///
		/// The main ownershipt of the asset changes
		/// else the call fails.
		///
		/// - `id`: The identifier of the asset
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 1))]
		pub fn claim_onwership(origin: OriginFor<T>, id: Identifier<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let asset_owner =
				Self::get_main_owner_by_asset(&id).ok_or(Error::<T>::AssetDoesNotExist)?;

			// Make sure that the origin is not the asset owner
			ensure!(asset_owner != who, Error::<T>::AlreadyMainOnwer);

			match ProportionalAssetToOwnerToMetadata::<T>::get(id, who.clone()) {
				None => Err(Error::<T>::NotEnoughShares)?,
				Some(origin_metadata) => {
					// Make sure that origin has 50% of the shares
					ensure!(origin_metadata.shares.gt(&50), Error::<T>::NotEnoughShares);

					Self::set_main_owner(who.clone(), &id);

					Self::deposit_event(Event::MainOwnerSet(who, id));

					Ok(())
				},
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn account_id() -> T::AccountId {
		sp_runtime::traits::AccountIdConversion::into_account_truncating(&PALLET_ID)
	}

	fn is_owner_of(who: &T::AccountId, id: &Identifier<T>) -> bool {
		if let Some(owner) = ProportionalAssetToMainOwner::<T>::get(id) {
			if &owner == who {
				return true;
			}
		}
		false
	}

	fn get_main_owner_by_asset(id: &Identifier<T>) -> Option<T::AccountId> {
		ProportionalAssetToMainOwner::<T>::get(id)
	}

	fn set_main_owner(who: T::AccountId, id: &Identifier<T>) {
		ProportionalAssetToMainOwner::<T>::set(id, Some(who))
	}

	fn balance_to_u64_option(input: impl TryInto<u64>) -> Option<u64> {
		input.try_into().ok()
	}
}
