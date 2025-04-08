<p align="center">
  <a href="https://teachfi.network/" target="blank"><img src="https://teachfi.network/teachfi-logo.svg" width="256" alt="TeachFi Logo" /><br/>Educational Platform</a>
</p>
<br/>
<br/>
<br/>

# Description

The Platform SC is the back-bone of TeachFi's Educational Platform. It is a subscription based service that offers the possibility to any educational institution worldwide to offer its students the opportunity to learn modern blockchain-based financial instruments and much more. For each subscriber, the Platform SC will deploy a set of smart contracts with different applications:
- [NFT Marketplace](https://github.com/TeachFiNetwork/tfn-nft-marketplace-rs) - empowers students to monetize their artistic skills
- [Test Launchpad](https://github.com/TeachFiNetwork/tfn-test-launchpad-rs) - capital raising platform for the students entrepreneurial ideas
- [Test DEX](https://github.com/TeachFiNetwork/tfn-test-dex-rs) - decentralized market for the students' tokens with different usecases
- [Test Staking](https://github.com/TeachFiNetwork/tfn-test-staking-rs) - teach interest earning while helping students secure their blockchain projects

The subscriber will also have access to several online courses and an online shop for related educational materials.
<br/>
<br/>
<br/>
## Endpoints

<br/>

```rust
subscribe(identity_id: OptionalValue<u64>)
```
>[!IMPORTANT]
>*Requirements:* state = active, payment_token = governance_token, payment_amount = subscription_fee.*

>[!NOTE]
>If the caller is an already existing subscriber, its subscription's validity period is extended. Otherwise, the `identity_id` parameter is required in order to register a new subscriber.
For new subscribers, the SC deploys a new set of NFT Marketplace, Test Launchpad, Test DEX and Test Staking smart contracts with the same code as the template addresses.

>[!WARNING]
>It is mandatory that the `identity_id` exists in the [Digital Identity SC](https://github.com/TeachFiNetwork/tfn-digital-identity-rs).
<br/>

```rust
subscribeFranchise(franchise_address: ManagedAddress, identity_id: u64)
```
>[!IMPORTANT]
*Requirements:* state = active, caller = LaunchpadSC.

>[!NOTE]
>This endpoint is called by the LaunchpadSC when a new [FranchiseDAO SC](https://github.com/TeachFiNetwork/tfn-franchise-dao-rs) is deployed and it registers the franchise's address as a subscriber for free.
The SC deploys a new set of NFT Marketplace, Test Launchpad, Test DEX and Test Staking smart contracts with the same code as the template addresses.

<br/>

```rust
whitelistAddress(address: ManagedAddress)
```
>[!IMPORTANT]
*Requirements:* state = active, caller is a subscriber with a valid subscription, number of subscriber's whitelisted addresses is less than max_subscriber_addresses.

>[!NOTE]
>The address parameter is whitelisted so it can access the child contracts of the subscriber.

<br/>

```rust
removeAddress(address: ManagedAddress)
```
>[!IMPORTANT]
*Requirements:* state = active, caller is a subscriber.

>[!NOTE]
>The address sent as a parameter is removed from the list of whitelisted addresses and can no longer access the functionality of the subscriber's child contracts.

<br/>

```rust
upgradeLaunchpad(subscriber_address: Option<ManagedAddress>, arguments: OptionalValue<ManagedArgBuffer>)
```
>[!IMPORTANT]
*Requirements:* state = active, if the subscriber_address parameter is specified, the caller must be the SC owner or a DAO member.

>[!NOTE]
>Updates the code of the subscriber's child Test Launchpad SC with the template's code.

<br/>

```rust
upgradeDEX(subscriber_address: Option<ManagedAddress>, arguments: OptionalValue<ManagedArgBuffer>)
```
>[!IMPORTANT]
*Requirements:* state = active, if the subscriber_address parameter is specified, the caller must be the SC owner or a DAO member.

>[!NOTE]
>Updates the code of the subscriber's child Test DEX SC with the template's code.

<br/>

```rust
upgradeStaking(subscriber_address: Option<ManagedAddress>, arguments: OptionalValue<ManagedArgBuffer>)
```
>[!IMPORTANT]
*Requirements:* state = active, if the subscriber_address parameter is specified, the caller must be the SC owner or a DAO member.

>[!NOTE]
>Updates the code of the subscriber's child Test Staking SC with the template's code.

<br/>

```rust
upgradeNFTMarketplace(subscriber_address: Option<ManagedAddress>, arguments: OptionalValue<ManagedArgBuffer>)
```
>[!IMPORTANT]
*Requirements:* state = active, if the subscriber_address parameter is specified, the caller must be the SC owner or a DAO member.

>[!NOTE]
>Updates the code of the subscriber's child NFT Marketplace SC with the template's code.

<br/>

```rust
setStateActive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as active.

<br/>

```rust
setStateInactive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as inactive.

<br/>

```rust
setTemplateAddresses(
    template_test_launchpad: ManagedAddress,
    template_test_dex: ManagedAddress,
    template_test_staking: ManagedAddress,
    template_nft_marketplace: ManagedAddress
)
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>As the endpoint name suggests, it changes the addresses of the template smart contract addreses.

>[!CAUTION]
>However, this endpoint should basically never be called.

<br/>

## View functions

<br/>

```rust
getState() -> State
```
>Returns the state of the SC (Active or Inactive).

<br/>

```rust
getGovernanceToken() -> TokenIdentifier
```
>Returns the DAO's governance token, which is also the payment token of the subscription fee.

<br/>

```rust
getMainDAO() -> ManagedAddress
```
>Returns the address of the Main DAO SC.

<br/>

```rust
getDigitalIdentity() -> ManagedAddress
```
>Return the address of the Digital Identity SC.

<br/>

```rust
getTemplateTestLaunchpad() -> ManagedAddress
```
>Returns the address of the Template Test Launchpad SC.

<br/>

```rust
getTemplateTestDEX() -> ManagedAddress
```
>Returns the address of the Template Test DEX SC.

<br/>

```rust
getTemplateTestStaking() -> ManagedAddress
```
>Returns the address of the Template Test Staking SC.

<br/>

```rust
getTemplateNFTMarketplace() -> ManagedAddress
```
>Returns the address of the Template NFT Marketplace SC.

<br/>

```rust
getSubscriptionFee() -> BigUint
```
>Returns the subscription fee amount (to be payed in governance tokens).

<br/>

```rust
getSubscriptionPeriod() -> u64
```
>Returns the validity period of the subscription (default is 365 days).

<br/>

```rust
getMaxSubscriberAddresses() -> usize
```
>Returns the maximum number of addresses a subscriber can whitelist (default is 1000).

<br/>

```rust
getSubscriber(subscriber_id: u64) -> Subscriber
```
>Returns the Subscriber object corresponding to the subscriber_id parameter.

<br/>

```rust
getLastSubscriberId() -> u64
```
>Returns the `ID - 1` of the last registered subscriber.

<br/>

```rust
getWhitelistedAddresses(subscriber_id: u64) -> ManagedVec<ManagedAddress>
```
>Returns the whitelisted addresses of the subscriber identified by the subscriber_id parameter.

<br/>

```rust
getAllSubscribers(only_active: bool) -> ManagedVec<Subscriber>
```
>Returns either all or only the active subscribers (with a valid subscription, not expired), based on the value supplied for the only_active parameter.

<br/>

```rust
getSubscriberIdByAddress(address: ManagedAddress) -> Option<u64>
```
>Returns Some(subscriber_id) if a subscriber with the specified address parameter exists, and None otherwise.

<br/>

```rust
checkWhitelisted(address: ManagedAddress)
```
>Looks up the specified address parameter in the whitelisted users lists of all active subscribers. If not found, it will throw a "not whitelisted error".

<br/>

```rust
getSubscribersCount(only_active: bool) -> u64
```
>Returns either the total subscribers count or only the active subscribers count, based on the value supplied for the only_active parameter.

<br/>

```rust
getWhitelistedWalletsCount(only_active: bool) -> u64
```
>Returns either the total whitelisted addresses count or only the whitelisted users of active subscribers count, based on the value supplied for the only_active parameter.

<br/>

```rust
getAddressDetails(address: ManagedAddress) -> (Option<Subscriber>, ManagedVec<Subscriber>)
```
>The first result will contain Some(subscriber) if the specified address is a subscriber and None otherwise. The second result will contain the list of subscribers in which the specified address is whitelisted.

<br/>

```rust
getContractInfo() -> PlatformInfo
```
>This is an all-in-one endpoint returning several relevant SC informations: state, governance_token, subscription_fee, subscription_period, max_subscriber_addresses, subscribers_count, active_subscribers_count, whitelisted_wallets_count, active_whitelisted_wallets_count.

<br/>

## Custom types

<br/>

```rust
pub enum State {
    Inactive,
    Active,
}
```

<br/>

```rust
pub struct Subscriber<M: ManagedTypeApi> {
    pub id: u64,
    pub address: ManagedAddress<M>,
    pub identity_id: u64,
    pub launchpad_sc: ManagedAddress<M>,
    pub dex_sc: ManagedAddress<M>,
    pub staking_sc: ManagedAddress<M>,
    pub nft_marketplace_sc: ManagedAddress<M>,
    pub validity: u64,
}
```

<br/>

```rust
pub struct PlatformInfo<M: ManagedTypeApi> {
    pub state: State,
    pub governance_token: TokenIdentifier<M>,
    pub subscription_fee: BigUint<M>,
    pub subscription_period: u64,
    pub max_subscriber_addresses: usize,
    pub subscribers_count: u64,
    pub active_subscribers_count: u64,
    pub whitelisted_wallets_count: u64,
    pub active_whitelisted_wallets_count: u64,
}
```
