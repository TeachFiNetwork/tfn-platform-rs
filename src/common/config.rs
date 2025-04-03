multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::common::errors::*;
use tfn_dao::common::config::ProxyTrait as _;

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct SubscriberDetails<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub description: ManagedBuffer<M>,
    pub logo: ManagedBuffer<M>,
    pub card: ManagedBuffer<M>,
    pub website: ManagedBuffer<M>,
    pub email: ManagedBuffer<M>,
    pub twitter: ManagedBuffer<M>,
    pub telegram: ManagedBuffer<M>,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct Subscriber<M: ManagedTypeApi> {
    pub id: u64,
    pub address: ManagedAddress<M>,
    pub details: SubscriberDetails<M>,
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

    #[only_owner]
    #[endpoint(setMainDAO)]
    fn set_main_dao(&self, address: ManagedAddress) {
        require!(self.main_dao().is_empty(), ERROR_DAO_ALREADY_SET);

        self.main_dao().set(&address);
        let governance_token: TokenIdentifier = self.dao_contract_proxy()
            .contract(address)
            .governance_token()
            .execute_on_dest_context();
        self.governance_token().set(governance_token);
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

    // proxies
    #[proxy]
    fn dao_contract_proxy(&self) -> tfn_dao::Proxy<Self::Api>;
}
