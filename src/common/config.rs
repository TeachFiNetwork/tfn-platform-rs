multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::errors::*;
use crate::proxies::dao_proxy::{self};

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
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

#[multiversx_sc::module]
pub trait ConfigModule {
    // state
    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        require!(!self.main_dao().is_empty(), ERROR_DAO_NOT_SET);

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

    // CONTRACTS
    #[view(getMainDAO)]
    #[storage_mapper("main_dao")]
    fn main_dao(&self) -> SingleValueMapper<ManagedAddress>;

    // should be called only by the DAO SC at initialization
    #[endpoint(setMainDAO)]
    fn set_main_dao(&self) {
        require!(self.main_dao().is_empty(), ERROR_DAO_ALREADY_SET);

        let caller = self.blockchain().get_caller();
        self.main_dao().set(&caller);
        let governance_token: TokenIdentifier = self.dao_contract_proxy()
            .contract(caller)
            .governance_token()
            .execute_on_dest_context();
        self.governance_token().set(governance_token);
    }

    #[view(getDigitalIdentity)]
    #[storage_mapper("digital_identity")]
    fn digital_identity(&self) -> SingleValueMapper<ManagedAddress>;

    // should be called only by the DAO SC at initialization
    #[endpoint(setDigitalIdentity)]
    fn set_digital_identity(&self, address: ManagedAddress) {
        require!(self.digital_identity().is_empty(), ERROR_DIGITAL_IDENTITY_ALREADY_SET);

        self.digital_identity().set(address);
    }

    #[view(getTemplateTestLaunchpad)]
    #[storage_mapper("template_test_launchpad")]
    fn template_test_launchpad(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTemplateTestDEX)]
    #[storage_mapper("template_test_dex")]
    fn template_test_dex(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTemplateTestStaking)]
    #[storage_mapper("template_test_staking")]
    fn template_test_staking(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTemplateNFTMarketplace)]
    #[storage_mapper("template_nft_marketplace")]
    fn template_nft_marketplace(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setTemplateAddresses)]
    fn set_template_addresses(
        &self,
        template_test_launchpad: ManagedAddress,
        template_test_dex: ManagedAddress,
        template_test_staking: ManagedAddress,
        template_nft_marketplace: ManagedAddress,
    ) {
        self.template_test_launchpad().set(&template_test_launchpad);
        self.template_test_dex().set(&template_test_dex);
        self.template_test_staking().set(&template_test_staking);
        self.template_nft_marketplace().set(&template_nft_marketplace);
    }
    // END CONTRACTS

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
    fn subscribers(&self, id: u64) -> SingleValueMapper<Subscriber<Self::Api>>;

    #[view(getLastSubscriberId)]
    #[storage_mapper("last_subscriber_id")]
    fn last_subscriber_id(&self) -> SingleValueMapper<u64>;

    #[view(getWhitelistedAddresses)]
    #[storage_mapper("whitelisted_addresses")]
    fn whitelisted_addresses(&self, subscriber_id: u64) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getSubscribers)]
    fn get_subscribers(&self) -> ManagedVec<Subscriber<Self::Api>> {
        let mut subscribers = ManagedVec::new();
        for id in 0..self.last_subscriber_id().get() {
            subscribers.push(self.subscribers(id).get());
        }

        subscribers
    }

    #[view(getSubscriberIdByAddress)]
    fn get_subscriber_id_by_address(&self, address: &ManagedAddress) -> Option<u64> {
        (0..self.last_subscriber_id().get()).find(|&i| &self.subscribers(i).get().address == address)
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

            if self.subscribers(subscriber_id).get().validity > current_time {
                return
            }
        }

        sc_panic!(ERROR_NOT_WHITELISTED)
    }

    #[view(getActiveSubscribersCount)]
    fn get_active_subscribers_count(&self) -> usize {
        let current_time = self.blockchain().get_block_timestamp();
        let mut count = 0;
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).get().validity > current_time {
                count += 1;
            }
        }

        count
    }

    #[view(getWhitelistedWalletsCount)]
    fn get_whitelisted_wallets_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.last_subscriber_id().get() {
            count += self.whitelisted_addresses(i).len();
        }

        count
    }

    #[view(getActiveWhitelistedWalletsCount)]
    fn get_active_whitelisted_wallets_count(&self) -> usize {
        let current_time = self.blockchain().get_block_timestamp();
        let mut count = 0;
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).get().validity > current_time {
                count += self.whitelisted_addresses(id).len();
            }
        }

        count
    }

    #[view(getAddressDetails)]
    fn get_address_details(
        &self,
        address: ManagedAddress,
    ) -> (
        Option<Subscriber<Self::Api>>,
        ManagedVec<Self::Api, Subscriber<Self::Api>>, // user subscribers
    ) {
        let mut is_subscriber: Option<Subscriber<Self::Api>> = None;
        let mut subscriptions: ManagedVec<Self::Api, Subscriber<Self::Api>> = ManagedVec::new();
        for id in 0..self.last_subscriber_id().get() {
            let subscriber = self.subscribers(id).get();
            if self.whitelisted_addresses(id).contains(&address) {
                subscriptions.push(subscriber.clone());
            }
            if subscriber.address == address {
                is_subscriber = Some(subscriber);
            }
        }

        if is_subscriber.is_some() || !subscriptions.is_empty() {
            return (is_subscriber, subscriptions);
        }

        sc_panic!(ERROR_NOT_WHITELISTED)
    }

    // proxies
    #[proxy]
    fn dao_contract_proxy(&self) -> dao_proxy::Proxy<Self::Api>;
}
