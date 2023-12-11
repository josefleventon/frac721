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

