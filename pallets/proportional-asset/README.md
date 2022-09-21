# Proportional Asset pallet.

This is a pallet which provides functionality for managing a Proportional Asset. A proportional asset is owned by multiple users. Each user is able to offer shares of owned asset, buy other shares, transfer them to other account and finally claim main ownership of the asset.

## Interface

### Testing

Run the tests with:
`cargo test`

### Dispatchable Functions

Proportional Asset Module:

- `create_proportional_asset` - Create a new proportional asset providing descriptive data.
- `offer_shares` - Offers new shares for an asset for sale.
- `transfer_shares_to_account` - Transfer shares to an account.
- `buy_shares` - Buy offered shares
- `claim_onwership` - Claim the main ownership of an asset.

### Improvements

- Validate an asset before buying from a trusted party - use of did pallet
- Store currency in a pot - use of treasury pallet
- Keep track of stakeholders involved in each asset - membership pallet
- Vote for adding stakeholders for a specific asset depending the shares held - use of democracy pallet

### Alternative design

- This pallet could be handling only one asset and use as a generic uniqueness service provided from the runtime

### Example Applications

- Real estate: Sell tokenized houses
