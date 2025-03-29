multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::errors::*;

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    // state
    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        require!(!self.test_launchpad().is_empty(), ERROR_LAUNCHPAD_NOT_SET);
        require!(!self.test_dex().is_empty(), ERROR_DEX_NOT_SET);
        require!(!self.test_staking().is_empty(), ERROR_STAKING_NOT_SET);
        require!(!self.nft_marketplace().is_empty(), ERROR_NFT_MARKETPLACE_NOT_SET);

        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    // governance token
    #[view(getGovernanceToken)]
    #[storage_mapper("governance_token")]
    fn governance_token(&self) -> SingleValueMapper<TokenIdentifier>;

    // contracts
    #[view(getMainDao)]
    #[storage_mapper("main_dao")]
    fn main_dao(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTestLaunchpad)]
    #[storage_mapper("test_launchpad")]
    fn test_launchpad(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setTestLaunchpad)]
    fn set_test_launchpad(&self, address: ManagedAddress) {
        self.test_launchpad().set(address);
    }

    #[view(getTestDEX)]
    #[storage_mapper("test_dex")]
    fn test_dex(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setTestDEX)]
    fn set_test_dex(&self, address: ManagedAddress) {
        self.test_dex().set(address);
    }

    #[view(getTestStaking)]
    #[storage_mapper("test_staking")]
    fn test_staking(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setTestStaking)]
    fn set_test_staking(&self, address: ManagedAddress) {
        self.test_staking().set(address);
    }

    #[view(getNFTMarketplace)]
    #[storage_mapper("nft_marketplace")]
    fn nft_marketplace(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setNFTMarketplace)]
    fn set_nft_marketplace(&self, address: ManagedAddress) {
        self.nft_marketplace().set(address);
    }

    // subscription params
    #[view(getSubscriptionFee)]
    #[storage_mapper("subscription_fee")]
    fn subscription_fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getSubscriptionPeriod)] // days
    #[storage_mapper("subscription_period")]
    fn subscription_period(&self) -> SingleValueMapper<u64>;

    #[view(getMaxSubscriberAddresses)]
    #[storage_mapper("max_subscriber_addresses")]
    fn max_subscriber_addresses(&self) -> SingleValueMapper<usize>;

    // subscribers
    #[view(getSubscriber)]
    #[storage_mapper("subscribers")]
    fn subscribers(&self, id: u64) -> SingleValueMapper<ManagedAddress>;

    #[view(getLastSubscriberId)]
    #[storage_mapper("last_subscriber_id")]
    fn last_subscriber_id(&self) -> SingleValueMapper<u64>;

    #[view(getSubscriptionValidity)]
    #[storage_mapper("subscription_validity")]
    fn subscription_validity(&self, id: u64) -> SingleValueMapper<u64>;

    #[view(getWhitelistedAddresses)]
    #[storage_mapper("whitelisted_addresses")]
    fn whitelisted_addresses(&self, subscriber_id: u64) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getSubscribers)]
    fn get_subscribers(&self) -> ManagedVec<ManagedAddress> {
        let mut subscribers: ManagedVec<ManagedAddress> = ManagedVec::new();
        for i in 0..self.last_subscriber_id().get() {
            let subscriber = self.subscribers(i).get();
            if subscriber != ManagedAddress::zero() {
                subscribers.push(subscriber);
            }
        }

        subscribers
    }

    #[view(getSubscriberIdByAddress)]
    fn get_subscriber_id_by_address(&self, address: &ManagedAddress) -> Option<u64> {
        (0..self.last_subscriber_id().get()).find(|&i| &self.subscribers(i).get() == address)
    }

    #[view(checkWhitelisted)]
    fn check_whitelisted(
        &self,
        address: ManagedAddress,
    ) {
        let current_time = self.blockchain().get_block_timestamp();
        for subscriber_id in 0..self.last_subscriber_id().get() {
            if !self.whitelisted_addresses(subscriber_id).contains(&address) {
                continue
            }

            if self.subscription_validity(subscriber_id).get() > current_time {
                return
            }
        }

        sc_panic!(ERROR_NOT_WHITELISTED)
    }
}
