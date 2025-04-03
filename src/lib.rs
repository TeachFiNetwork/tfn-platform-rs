#![no_std]

multiversx_sc::imports!();

pub mod common;

use common::{config::*, consts::*, errors::*};
use tfn_dao::{common::{config::ProxyTrait as _, errors::ERROR_ONLY_LAUNCHPAD}, ProxyTrait as _};

#[multiversx_sc::contract]
pub trait TFNPlatformContract<ContractReader>:
common::config::ConfigModule
{
    #[init]
    fn init(
        &self,
        template_test_launchpad: ManagedAddress,
        template_test_dex: ManagedAddress,
        template_test_staking: ManagedAddress,
        template_nft_marketplace: ManagedAddress,
    ) {
        self.subscription_period().set(DEFAULT_VALIDITY);
        self.max_subscriber_addresses().set(DEFAULT_MAX_SUBSCRIBERS);

        self.template_test_launchpad().set(&template_test_launchpad);
        self.template_test_dex().set(&template_test_dex);
        self.template_test_staking().set(&template_test_staking);
        self.template_nft_marketplace().set(&template_nft_marketplace);
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    #[payable("*")]
    #[endpoint(subscribe)]
    fn subscribe(
        &self,
        details: OptionalValue<SubscriberDetails<Self::Api>>,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        require!(!self.is_franchise(&caller), ERROR_NOT_ALLOWED);

        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.governance_token().get(), ERROR_WRONG_PAYMENT_TOKEN);
        require!(payment.amount == self.subscription_fee().get(), ERROR_WRONG_PAYMENT_AMOUNT);

        let mut subscriber = match self.get_subscriber_id_by_address(&caller) {
            Some(subscriber_id) => self.subscribers(subscriber_id).get(),
            None => {
                require!(details.is_some(), ERROR_NO_DETAILS);

                let subscriber_id = self.last_subscriber_id().get();
                self.last_subscriber_id().set(subscriber_id + 1);
                let subscriber = Subscriber {
                    id: subscriber_id,
                    address: caller,
                    details: details.into_option().unwrap(),
                    validity: 0,
                };
                self.subscribers(subscriber_id).set(&subscriber);

                subscriber
            }
        };
        let current_time = self.blockchain().get_block_timestamp();
        if subscriber.validity < current_time {
            subscriber.validity = current_time + self.subscription_period().get();
        } else {
            subscriber.validity += self.subscription_period().get();
        }
        self.subscribers(subscriber.id).set(subscriber);

        self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .add_funds()
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    #[endpoint(changeDetails)]
    fn change_details(
        &self,
        new_details: SubscriberDetails<Self::Api>,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_id = match self.get_subscriber_id_by_address(&caller) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let mut subscriber = self.subscribers(subscriber_id).get();
        subscriber.details = new_details;
        self.subscribers(subscriber_id).set(subscriber);
    }

    #[endpoint(subscribeFranchise)]
    fn subscribe_franchise(
        &self,
        franchise_address: ManagedAddress,
        details: SubscriberDetails<Self::Api>,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let launchpad: ManagedAddress = self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .launchpad_sc()
            .execute_on_dest_context();
        require!(self.blockchain().get_caller() == launchpad, ERROR_ONLY_LAUNCHPAD);
        require!(self.get_subscriber_id_by_address(&franchise_address).is_none(), ERROR_ALREADY_SUBSCRIBED);

        let subscriber_id = self.last_subscriber_id().get();
        self.last_subscriber_id().set(subscriber_id + 1);
        let subscriber = Subscriber {
            id: subscriber_id,
            address: franchise_address,
            details,
            validity: self.blockchain().get_block_timestamp() + VALIDITY_FOR_FRANCHISE,
        };
        self.subscribers(subscriber_id).set(subscriber);
    }

    #[endpoint(whitelistAddress)]
    fn whitelist_address(
        &self,
        address: ManagedAddress,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_id = match self.get_subscriber_id_by_address(&caller) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let current_time = self.blockchain().get_block_timestamp();
        require!(self.subscribers(subscriber_id).get().validity > current_time, ERROR_SUBSCRIPTION_EXPIRED);
        require!(self.whitelisted_addresses(subscriber_id).len() < self.max_subscriber_addresses().get(), ERROR_TOO_MANY_ADDRESSES);

        self.whitelisted_addresses(subscriber_id).insert(address);
    }

    #[endpoint(removeAddress)]
    fn remove_address(
        &self,
        address: ManagedAddress,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_id = match self.get_subscriber_id_by_address(&caller) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };

        self.whitelisted_addresses(subscriber_id).swap_remove(&address);
    }

    // helpers
    fn is_franchise(&self, address: &ManagedAddress) -> bool {
        self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .is_franchise(address)
            .execute_on_dest_context()
    }
}
