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

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
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
        if !self.main_dao().is_empty() {
            return
        }

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
        if !self.digital_identity().is_empty() {
            return
        }

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

    #[only_owner]
    #[endpoint(setSubscriptionFee)]
    fn set_subscription_fee(&self, new_fee: BigUint) {
        self.subscription_fee().set(new_fee);
    }

    #[view(getSubscriptionPeriod)] // seconds
    #[storage_mapper("subscription_period")]
    fn subscription_period(&self) -> SingleValueMapper<u64>;

    #[only_owner]
    #[endpoint(setSubscriptionPeriod)]
    fn set_subscription_period(&self, new_period: u64) {
        self.subscription_period().set(new_period);
    }

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

    #[storage_mapper("whitelisted_addresses")]
    fn whitelisted_addresses(&self, subscriber_id: u64) -> UnorderedSetMapper<ManagedAddress>;

    #[view(getWhitelistedAddresses)]
    fn get_whitelisted_addresses(&self, subscriber_id: u64) -> ManagedVec<ManagedAddress<Self::Api>> {
        let mut addresses = ManagedVec::new();
        for address in self.whitelisted_addresses(subscriber_id).iter() {
            addresses.push(address);
        }

        addresses
    }

    #[view(getAllSubscribers)]
    fn get_all_subscribers(&self, only_active: bool) -> ManagedVec<Subscriber<Self::Api>> {
        let current_time = self.blockchain().get_block_timestamp();
        let mut subscribers = ManagedVec::new();
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).is_empty() {
                continue
            }

            let subscriber = self.subscribers(id).get();
            if !only_active || subscriber.validity > current_time {
                subscribers.push(subscriber);
            }
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
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).is_empty() {
                continue
            }

            let subscriber = self.subscribers(id).get();
            let is_subscriber = address == subscriber.address;
            let is_user = self.whitelisted_addresses(id).contains(&address);
            if !is_user && !is_subscriber {
                continue
            }

            if subscriber.validity > current_time {
                return
            }
        }

        sc_panic!(ERROR_NOT_WHITELISTED)
    }

    #[view(getSubscribersCount)]
    fn get_subscribers_count(&self, only_active: bool) -> u64 {
        let current_time = self.blockchain().get_block_timestamp();
        let mut count = 0;
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).is_empty() {
                continue
            }

            if !only_active || self.subscribers(id).get().validity > current_time {
                count += 1;
            }
        }

        count
    }

    #[view(getWhitelistedWalletsCount)]
    fn get_whitelisted_wallets_count(&self, only_active: bool) -> u64 {
        let mut count = 0u64;
        let current_time = self.blockchain().get_block_timestamp();
        for id in 0..self.last_subscriber_id().get() {
            if self.subscribers(id).is_empty() {
                continue
            }

            if !only_active || self.subscribers(id).get().validity > current_time {
                count += self.whitelisted_addresses(id).len() as u64 + 1; // +1 for the subscriber itself
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
            if self.subscribers(id).is_empty() {
                continue
            }

            let subscriber = self.subscribers(id).get();
            if self.whitelisted_addresses(id).contains(&address) {
                subscriptions.push(subscriber.clone());
            }
            if subscriber.address == address {
                is_subscriber = Some(subscriber);
            }
        }

        (is_subscriber, subscriptions)
    }

    #[view(getContractInfo)]
    fn get_contract_info(&self) -> PlatformInfo<Self::Api> {
        PlatformInfo {
            state: self.state().get(),
            governance_token: self.governance_token().get(),
            subscription_fee: self.subscription_fee().get(),
            subscription_period: self.subscription_period().get(),
            max_subscriber_addresses: self.max_subscriber_addresses().get(),
            subscribers_count: self.get_subscribers_count(false),
            active_subscribers_count: self.get_subscribers_count(true),
            whitelisted_wallets_count: self.get_whitelisted_wallets_count(false),
            active_whitelisted_wallets_count: self.get_whitelisted_wallets_count(true),
        }
    }

    // proxies
    #[proxy]
    fn dao_contract_proxy(&self) -> dao_proxy::Proxy<Self::Api>;
}
