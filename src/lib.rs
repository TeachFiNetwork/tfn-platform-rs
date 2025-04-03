#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod proxies;

use common::{config::*, consts::*, errors::*};
use proxies::{dao_proxy::ProxyTrait as _, template_proxy::{self}};

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

    fn deploy_contracts(&self) -> (ManagedAddress, ManagedAddress, ManagedAddress, ManagedAddress) {
        let (launchpad_address, ()) = self
            .template_contract_proxy()
            .init()
            .deploy_from_source(
                &self.template_test_launchpad().get(),
                CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC,
            );
        let (dex_address, ()) = self
            .template_contract_proxy()
            .init()
            .deploy_from_source(
                &self.template_test_dex().get(),
                CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC,
            );
        let (staking_address, ()) = self
            .template_contract_proxy()
            .init()
            .deploy_from_source(
                &self.template_test_staking().get(),
                CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC,
            );
        let (nft_marketplace_address, ()) = self
            .template_contract_proxy()
            .init()
            .deploy_from_source(
                &self.template_nft_marketplace().get(),
                CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC,
            );

        (launchpad_address, dex_address, staking_address, nft_marketplace_address)
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

                let (launchpad_sc, dex_sc, staking_sc, nft_marketplace_sc) = self.deploy_contracts();
                let subscriber_id = self.last_subscriber_id().get();
                self.last_subscriber_id().set(subscriber_id + 1);
                let subscriber = Subscriber {
                    id: subscriber_id,
                    address: caller,
                    details: details.into_option().unwrap(),
                    launchpad_sc,
                    staking_sc,
                    dex_sc,
                    nft_marketplace_sc,
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
        opt_subscriber_address: OptionalValue<ManagedAddress>,
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_address = match opt_subscriber_address {
            OptionalValue::Some(address) => {
                require!(address == self.blockchain().get_owner_address() || address == caller, ERROR_NOT_ALLOWED);

                address
            },
            OptionalValue::None => caller
        };
        let subscriber_id = match self.get_subscriber_id_by_address(&subscriber_address) {
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

        let (launchpad_sc, dex_sc, staking_sc, nft_marketplace_sc) = self.deploy_contracts();
        let subscriber_id = self.last_subscriber_id().get();
        self.last_subscriber_id().set(subscriber_id + 1);
        let subscriber = Subscriber {
            id: subscriber_id,
            address: franchise_address,
            details,
            launchpad_sc,
            staking_sc,
            dex_sc,
            nft_marketplace_sc,
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

    #[endpoint(upgradeLaunchpad)]
    fn upgrade_launchpad(
        &self,
        opt_subscriber_address: Option<ManagedAddress>,
        args: OptionalValue<ManagedArgBuffer<Self::Api>>
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_address = match opt_subscriber_address {
            Some(address) => {
                require!(address == self.blockchain().get_owner_address() || address == caller, ERROR_NOT_ALLOWED);

                address
            },
            None => caller
        };
        let subscriber_id = match self.get_subscriber_id_by_address(&subscriber_address) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let upgrade_args = match args {
            OptionalValue::Some(args) => args,
            OptionalValue::None => ManagedArgBuffer::new(),            
        };
        let subscriber = self.subscribers(subscriber_id).get();
        self.tx()
            .to(subscriber.launchpad_sc)
            .gas(self.blockchain().get_gas_left())
            .raw_upgrade()
            .arguments_raw(upgrade_args)
            .from_source(self.template_test_launchpad().get())
            .code_metadata(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC)
            .upgrade_async_call_and_exit();
    }

    #[endpoint(upgradeDEX)]
    fn upgrade_dex(
        &self,
        opt_subscriber_address: Option<ManagedAddress>,
        args: OptionalValue<ManagedArgBuffer<Self::Api>>
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_address = match opt_subscriber_address {
            Some(address) => {
                require!(address == self.blockchain().get_owner_address() || address == caller, ERROR_NOT_ALLOWED);

                address
            },
            None => caller
        };
        let subscriber_id = match self.get_subscriber_id_by_address(&subscriber_address) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let upgrade_args = match args {
            OptionalValue::Some(args) => args,
            OptionalValue::None => ManagedArgBuffer::new(),            
        };
        let subscriber = self.subscribers(subscriber_id).get();
        self.tx()
            .to(subscriber.dex_sc)
            .gas(self.blockchain().get_gas_left())
            .raw_upgrade()
            .arguments_raw(upgrade_args)
            .from_source(self.template_test_dex().get())
            .code_metadata(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC)
            .upgrade_async_call_and_exit();
    }

    #[endpoint(upgradeStaking)]
    fn upgrade_staking(
        &self,
        opt_subscriber_address: Option<ManagedAddress>,
        args: OptionalValue<ManagedArgBuffer<Self::Api>>
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_address = match opt_subscriber_address {
            Some(address) => {
                require!(address == self.blockchain().get_owner_address() || address == caller, ERROR_NOT_ALLOWED);

                address
            },
            None => caller
        };
        let subscriber_id = match self.get_subscriber_id_by_address(&subscriber_address) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let upgrade_args = match args {
            OptionalValue::Some(args) => args,
            OptionalValue::None => ManagedArgBuffer::new(),            
        };
        let subscriber = self.subscribers(subscriber_id).get();
        self.tx()
            .to(subscriber.staking_sc)
            .gas(self.blockchain().get_gas_left())
            .raw_upgrade()
            .arguments_raw(upgrade_args)
            .from_source(self.template_test_staking().get())
            .code_metadata(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC)
            .upgrade_async_call_and_exit();
    }

    #[endpoint(upgradeNFTMarketplace)]
    fn upgrade_nft_marketplace(
        &self,
        opt_subscriber_address: Option<ManagedAddress>,
        args: OptionalValue<ManagedArgBuffer<Self::Api>>
    ) {
        require!(self.state().get() == State::Active, ERROR_STATE_INACTIVE);

        let caller = self.blockchain().get_caller();
        let subscriber_address = match opt_subscriber_address {
            Some(address) => {
                require!(address == self.blockchain().get_owner_address() || address == caller, ERROR_NOT_ALLOWED);

                address
            },
            None => caller
        };
        let subscriber_id = match self.get_subscriber_id_by_address(&subscriber_address) {
            Some(subscriber_id) => subscriber_id,
            None => sc_panic!(ERROR_NOT_SUBSCRIBED)
        };
        let upgrade_args = match args {
            OptionalValue::Some(args) => args,
            OptionalValue::None => ManagedArgBuffer::new(),            
        };
        let subscriber = self.subscribers(subscriber_id).get();
        self.tx()
            .to(subscriber.nft_marketplace_sc)
            .gas(self.blockchain().get_gas_left())
            .raw_upgrade()
            .arguments_raw(upgrade_args)
            .from_source(self.template_nft_marketplace().get())
            .code_metadata(CodeMetadata::UPGRADEABLE | CodeMetadata::READABLE | CodeMetadata::PAYABLE_BY_SC)
            .upgrade_async_call_and_exit();
    }

    // helpers
    fn is_franchise(&self, address: &ManagedAddress) -> bool {
        self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .is_franchise(address)
            .execute_on_dest_context()
    }

    // proxies
    #[proxy]
    fn template_contract_proxy(&self) -> template_proxy::Proxy<Self::Api>;
}
