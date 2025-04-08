<p align="center">
  <a href="https://teachfi.network/" target="blank"><img src="https://teachfi.network/teachfi-logo.svg" width="120" alt="TeachFi Logo" /></a>
</p>

## Description

The Platform SC is the back-bone of TeachFi's Educational Platform.

## Endpoints

- subscribe(identity_id: u64)
- subscribeFranchise(franchise_address: Address, identity_id: u64)
- whitelistAddress(address: Address)
- removeAddress(address: Address)
- upgradeLaunchpad(subscriber_address: Address, arguments: ArgBuffer)
- upgradeDEX(subscriber_address: Address, arguments: ArgBuffer)
- upgradeStaking(subscriber_address: Address, arguments: ArgBuffer)
- upgradeNFTMarketplace(subscriber_address: Address, arguments: ArgBuffer)
- setStateActive() (only_owner)
- setStateInactive() (only_owner)
- setTemplateAddresses() (only_owner)

## View functions

- getState
- getGovernanceToken
- getMainDAO
- getDigitalIdentity
- getTemplateTestLaunchpad
- getTemplateTestDEX
- getTemplateTestStaking
- getTemplateNFTMarketplace
- getSubscriptionFee
- getSubscriptionPeriod
- getMaxSubscriberAddresses
- getSubscriber(subscriber_id: u64)

