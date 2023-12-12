# frac721

Frac721 is a CosmWasm smart contract designed for fractionalizing NFTs from specific SG721 collections, allowing users to deposit NFTs, receive tradable CW20 tokens representing fractional ownership, and reclaim NFTs from a vault.

## Core functionalities

1. **NFT Depositing**
   - Users can deposit NFTs from a specified SG721 collection into a vault managed by the contract.

2. **Token Generation**
   - Upon depositing an NFT, the contract issues a CW20 token to the user. This token represents fractional ownership of the deposited NFT.

3. **Claiming NFTs**
   - Users can claim any NFT from the vault by paying with one CW20 token. When a token is used for claiming, it is burned, removing it from circulation.

4. **Fee for Fractionalization**
   - Users are charged a fee for fractionalizing their NFT. The type of coin used for this fee is set in the contract's configuration.

These functionalities enable users to fractionalize ownership of NFTs, trade fractional shares, and reclaim NFTs from the vault.

## Terminology

### Vault

"Vault" describes the entirety of the NFTs stored in the contract.

### Depositing

"Depositing" describes the action of trading in 1 NFT in exchange for 1 token to be minted.

### Claiming

"Claiming" describes the action of trading in 1 token in exchange for 1 NFT out of the vault.

## Contract instantiation

The contract is instantiated by calling `Instantiate{collection_addr, cw20_config}`. `cw20_config` is used to instantiate a new CW20 contract.

Upon instantiation, the contract instantiates a new CW20 contract that will be used to issue tokens when NFTs are fractionalized.

The instantiator of the Frac721 contract will need to execute the `SetCW20Address{cw20_address}` message for the contract to begin accepting deposits, by providing the contract address of the CW20 contract that was instantiated.

## Contract methods

### `Deposit{}` (CW721 Send)

The Deposit method extends CW721Receiver and issues a singular token to the message sender in exchange for their NFT.

### `Claim{token_id}` (CW20 Send)

The Claim method requires for a singular CW20 token to be sent and sends the requested NFT to the message sender. The contract will subsequently burn the CW20 token


## Storage

### Deposited NFTs

Deposited NFTs are stored as follows:

```rust
pub struct VaultItem {
   pub token_id: String,
   pub depositor: Addr,
}
```
