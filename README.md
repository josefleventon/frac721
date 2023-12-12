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

## Contract instantiation

The contract is instantiated by calling `Instantiate{collection_addr}`. The collection address is stored in `CONFIG`.

Upon instantiation, the contract instantiates a new CW20 contract that will be used to issue tokens when NFTs are fractionalized.

## Contract methods

### `Deposit{}` (CW721 Send)

The Deposit method extends CW721Receiver and issues a singular token to the message sender in exchange for their NFT.

### `Claim{token_id}` (CW20 Send)

The Claim method requires for a singular CW20 token to be sent and sends the requested NFT to the message sender. The contract will subsequently burn the CW20 token

### ~~`UpdateConfig{collection_addr}`~~

The `UpdateConfig` method is privileged to the administrator of the contract.

> 💡 We should not allow for the collection address to be updated. Frac721 users should instantiate a new contract if they want to allow for fractionalization of a different collection.


## Storage

### Deposited NFTs

Deposited NFTs are stored in `deposits` as follows:

```rust
pub struct TokenDeposit {
   pub token_id: String,
   pub depositor: Addr,
}
```

