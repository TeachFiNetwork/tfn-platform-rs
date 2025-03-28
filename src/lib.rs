#![no_std]

multiversx_sc::imports!();

pub mod common;
pub mod helpers;

use common::{config::*, errors::*};

#[multiversx_sc::contract]
pub trait TFNPlatformContract<ContractReader>:
    common::config::ConfigModule
{
    #[init]
    fn init(
        &self,
        main_dao: ManagedAddress,
    ) {
        self.main_dao().set(main_dao);
    }

    #[upgrade]
    fn upgrade(&self) {
    }
}
