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
        main_dao: ManagedAddress,
    ) {
        self.main_dao().set(&main_dao);
        let governance_token: TokenIdentifier = self.dao_contract_proxy()
            .contract(main_dao)
            .governance_token()
            .execute_on_dest_context();
        self.governance_token().set(governance_token);
        self.subscription_period().set(DEFAULT_VALIDITY);
        self.max_subscriber_addresses().set(DEFAULT_MAX_SUBSCRIBERS);
    }

    #[upgrade]
    fn upgrade(&self) {
    }

    #[payable("*")]
    #[endpoint(subscribe)]
    fn subscribe(&self) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        require!(!self.is_franchise(&caller), ERROR_NOT_ALLOWED);

        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.governance_token().get(), ERROR_WRONG_PAYMENT_TOKEN);
        require!(payment.amount == self.subscription_fee().get(), ERROR_WRONG_PAYMENT_AMOUNT);

        let subscriber_id = match self.get_subscriber_id_by_address(&caller) {
            Some(subscriber_id) => subscriber_id,
            None => {
                let subscriber_id = self.last_subscriber_id().get();
                self.last_subscriber_id().set(subscriber_id + 1);
                self.subscribers(subscriber_id).set(&caller);

                subscriber_id
            }
        };
        let current_time = self.blockchain().get_block_timestamp();
        let mut validity = self.subscription_validity(subscriber_id).get();
            if validity < current_time {
            validity = current_time + self.subscription_period().get();
        } else {
            validity += self.subscription_period().get();
        }
        self.subscription_validity(subscriber_id).set(validity);

        self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .add_funds()
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<()>();
    }

    #[endpoint(subscribeFranchise)]
    fn subscribe_franchise(
        &self,
        franchise_address: ManagedAddress,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);
        // require!(self.is_franchise(&franchise_address), ERROR_ONLY_FRANCHISE);

        let launchpad: ManagedAddress = self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .launchpad_sc()
            .execute_on_dest_context();
        require!(self.blockchain().get_caller() == launchpad, ERROR_ONLY_LAUNCHPAD);

        let subscriber_id = match self.get_subscriber_id_by_address(&franchise_address) {
            Some(_) => sc_panic!(ERROR_ALREADY_SUBSCRIBED),
            None => {
                let subscriber_id = self.last_subscriber_id().get();
                self.last_subscriber_id().set(subscriber_id + 1);
                self.subscribers(subscriber_id).set(&franchise_address);

                subscriber_id
            }
        };
        self.subscription_validity(subscriber_id).set(self.blockchain().get_block_timestamp() + VALIDITY_FOR_FRANCHISE);
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
        require!(self.subscription_validity(subscriber_id).get() > current_time, ERROR_SUBSCRIPTION_EXPIRED);
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
        let franchises: ManagedVec<ManagedAddress> = self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .franchises()
            .execute_on_dest_context();
        for franchise in franchises.into_iter() {
            if &franchise == address {
                return true;
            }
        }

        false
    }

    // proxies
    #[proxy]
    fn dao_contract_proxy(&self) -> tfn_dao::Proxy<Self::Api>;
}
