use crate::{mock::*, Error, ProportionalAssetToOwnerToMetadata, TOTAL_SUPPLY};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_proportional_asset_success() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::ProportionalAssetInitialized(id, 1));
		assert_eq!(System::events()[0].event, expected_event);

		let owner_id = ProportionalAssetModule::get_main_owner_by_asset(&id).unwrap();
		assert_eq!(owner_id, 1);

		let owner_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, owner_id).unwrap();

		assert_eq!(owner_metadata.offers, 0);
		assert_eq!(owner_metadata.shares, TOTAL_SUPPLY);
		assert_eq!(owner_metadata.price, share_price);
	});
}

#[test]
fn create_proportional_asset_failure_duplicate() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		assert_noop!(
			ProportionalAssetModule::create_proportional_asset(
				Origin::signed(1),
				data,
				share_price
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn offer_shares_success() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 20;
		assert_ok!(ProportionalAssetModule::offer_shares(
			Origin::signed(1),
			id,
			offers,
			new_share_price
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesOffered(id, new_share_price));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();

		assert_eq!(stored_metadata.offers, offers);
		assert_eq!(new_share_price, stored_metadata.price);
	})
}

#[test]
fn offer_shares_failure_invalid_offers() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 101;

		let new_share_price = 20;
		assert_noop!(
			ProportionalAssetModule::offer_shares(Origin::signed(1), id, offers, new_share_price),
			Error::<Test>::InvalidOffers
		);
	})
}

#[test]
fn offer_shares_failure_different_account() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 20;
		assert_noop!(
			ProportionalAssetModule::offer_shares(Origin::signed(2), id, offers, new_share_price),
			Error::<Test>::NotMainOwner
		);
	})
}

#[test]
fn transfer_shares_to_account_success() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 50;

		assert_ok!(ProportionalAssetModule::transfer_shares_to_account(
			Origin::signed(1),
			id,
			amount,
			2
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, amount));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata_1.shares, 100 - amount);

		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();
		assert_eq!(stored_metadata_2.shares, amount);
	})
}

#[test]
fn transfer_shares_to_account_failure_invalid_account() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 50;

		assert_noop!(
			ProportionalAssetModule::transfer_shares_to_account(Origin::signed(2), id, amount, 2),
			Error::<Test>::InvalidAccount
		);
	})
}

#[test]
fn transfer_shares_to_account_failure_incorrect_share_selection() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 101;

		assert_noop!(
			ProportionalAssetModule::transfer_shares_to_account(Origin::signed(1), id, amount, 2),
			Error::<Test>::IncorrectSharesSelection
		);
	})
}

#[test]
fn buy_shares_success() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 20;
		assert_ok!(ProportionalAssetModule::offer_shares(
			Origin::signed(1),
			id,
			offers,
			new_share_price
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesOffered(id, new_share_price));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata.offers, offers);
		assert_eq!(stored_metadata.price, new_share_price);

		let shares_to_buy = 2;
		let amount_to_be_transferred = new_share_price.checked_mul(shares_to_buy).unwrap();
		assert_ok!(ProportionalAssetModule::buy_shares(
			Origin::signed(2),
			id,
			shares_to_buy,
			amount_to_be_transferred.into(),
			1
		));

		let expected_event_pallet =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, shares_to_buy));
		assert_eq!(System::events()[3].event, expected_event_pallet);

		let initial_balances = get_initial_balances();
		let initial_balance_1 = initial_balances[0].1;
		let initial_balance_2 = initial_balances[1].1;

		let new_balance_1 = Balances::free_balance(1);
		let new_balance_2 = Balances::free_balance(2);

		assert_eq!(
			new_balance_1,
			initial_balance_1.checked_add(amount_to_be_transferred.into()).unwrap()
		);
		assert_eq!(
			new_balance_2,
			initial_balance_2.checked_sub(amount_to_be_transferred.into()).unwrap()
		);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();

		assert_eq!(stored_metadata_1.shares, 100 - shares_to_buy);
		assert_eq!(stored_metadata_2.shares, shares_to_buy);

		// TODO: Check offers
	})
}

#[test]
fn buy_shares_failure_incorrect_seller_same_owner() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 20;
		assert_ok!(ProportionalAssetModule::offer_shares(
			Origin::signed(1),
			id,
			offers,
			new_share_price
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesOffered(id, new_share_price));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata.offers, offers);
		assert_eq!(stored_metadata.price, new_share_price);

		let shares_to_buy = 2;
		let amount_to_be_transferred = new_share_price.checked_mul(shares_to_buy).unwrap();
		assert_noop!(
			ProportionalAssetModule::buy_shares(
				Origin::signed(1),
				id,
				shares_to_buy,
				amount_to_be_transferred.into(),
				1
			),
			Error::<Test>::IncorrectSeller
		);
	})
}

#[test]
fn buy_shares_failure_incorrect_seller_not_owner() {
	new_test_ext().execute_with(|| {
		let share_price = 10;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 20;
		assert_ok!(ProportionalAssetModule::offer_shares(
			Origin::signed(1),
			id,
			offers,
			new_share_price
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesOffered(id, new_share_price));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata.offers, offers);
		assert_eq!(stored_metadata.price, new_share_price);

		let shares_to_buy = 2;
		let amount_to_be_transferred = new_share_price.checked_mul(shares_to_buy).unwrap();

		let src1: Vec<char> = vec!['a', 'b', '"', 'i', 'm', 'm', 'y', '"', '}'];
		let data: Vec<u8> = src1.iter().map(|c| *c as u8).collect::<Vec<_>>();
		let id = get_hash_from_vec(data);
		assert_noop!(
			ProportionalAssetModule::buy_shares(
				Origin::signed(2),
				id,
				shares_to_buy,
				amount_to_be_transferred.into(),
				1
			),
			Error::<Test>::IncorrectSeller
		);
	})
}

#[test]
fn buy_shares_failure_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let share_price = 100;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let offers = 5;

		let new_share_price = 100;
		assert_ok!(ProportionalAssetModule::offer_shares(
			Origin::signed(1),
			id,
			offers,
			new_share_price
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesOffered(id, new_share_price));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata.offers, offers);
		assert_eq!(stored_metadata.price, new_share_price);

		let shares_to_buy = 2;
		let amount_to_be_transferred = new_share_price.checked_mul(shares_to_buy).unwrap();
		assert_noop!(
			ProportionalAssetModule::buy_shares(
				Origin::signed(2),
				id,
				shares_to_buy,
				amount_to_be_transferred.into(),
				1
			),
			Error::<Test>::InsufficientBalance
		);
	})
}

#[test]
fn claim_onwership_success() {
	new_test_ext().execute_with(|| {
		let share_price = 1;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 51;

		assert_ok!(ProportionalAssetModule::transfer_shares_to_account(
			Origin::signed(1),
			id,
			amount,
			2
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, amount));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata_1.shares, 100 - amount);

		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();
		assert_eq!(stored_metadata_2.shares, amount);

		assert_ok!(ProportionalAssetModule::claim_onwership(Origin::signed(2), id));

		let expected_main_owner_event =
			Event::ProportionalAssetModule(crate::Event::MainOwnerSet(2, id));
		assert_eq!(System::events()[2].event, expected_main_owner_event);

		let is_owner = ProportionalAssetModule::is_owner_of(&2, &id);
		assert!(is_owner);
	})
}

#[test]
fn claim_onwership_failure_asset_does_not_exist() {
	new_test_ext().execute_with(|| {
		let share_price = 1;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 51;

		assert_ok!(ProportionalAssetModule::transfer_shares_to_account(
			Origin::signed(1),
			id,
			amount,
			2
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, amount));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata_1.shares, 100 - amount);

		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();
		assert_eq!(stored_metadata_2.shares, amount);

		let src1: Vec<char> = vec!['a', 'b', '"', 'i', 'm', 'm', 'y', '"', '}'];
		let data: Vec<u8> = src1.iter().map(|c| *c as u8).collect::<Vec<_>>();
		let hash_bytes = sp_io::hashing::blake2_256(&data);
		let wrong_id = sp_core::H256(hash_bytes);

		assert_noop!(
			ProportionalAssetModule::claim_onwership(Origin::signed(2), wrong_id),
			Error::<Test>::AssetDoesNotExist
		);
	})
}

#[test]
fn claim_onwership_failure_already_main_owner() {
	new_test_ext().execute_with(|| {
		let share_price = 1;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 51;

		assert_ok!(ProportionalAssetModule::transfer_shares_to_account(
			Origin::signed(1),
			id,
			amount,
			2
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, amount));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata_1.shares, 100 - amount);

		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();
		assert_eq!(stored_metadata_2.shares, amount);

		assert_noop!(
			ProportionalAssetModule::claim_onwership(Origin::signed(1), id),
			Error::<Test>::AlreadyMainOnwer
		);
	})
}

#[test]
fn claim_onwership_failure_not_enough_shares() {
	new_test_ext().execute_with(|| {
		let share_price = 1;

		let data = get_test_data();

		assert_ok!(ProportionalAssetModule::create_proportional_asset(
			Origin::signed(1),
			data.clone(),
			share_price
		));

		let id = get_hash_from_vec(data);

		let amount = 49;

		assert_ok!(ProportionalAssetModule::transfer_shares_to_account(
			Origin::signed(1),
			id,
			amount,
			2
		));

		let expected_event =
			Event::ProportionalAssetModule(crate::Event::SharesTransferred(1, 2, amount));
		assert_eq!(System::events()[1].event, expected_event);

		let stored_metadata_1 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &1).unwrap();
		assert_eq!(stored_metadata_1.shares, 100 - amount);

		let stored_metadata_2 = ProportionalAssetToOwnerToMetadata::<Test>::get(id, &2).unwrap();
		assert_eq!(stored_metadata_2.shares, amount);

		assert_noop!(
			ProportionalAssetModule::claim_onwership(Origin::signed(2), id),
			Error::<Test>::NotEnoughShares
		);
	})
}
