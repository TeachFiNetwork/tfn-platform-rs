multiversx_sc::imports!();

use tfn_dao::common::config::ProxyTrait as _;
use tfn_franchise_dao::common::school_config::{ProxyTrait as _, *};

use crate::common::config;

#[multiversx_sc::module]
pub trait HelpersModule:
config::ConfigModule
{
    fn is_franchise(&self, address: ManagedAddress) -> bool {
        let franchises: ManagedVec<ManagedAddress> = self.dao_contract_proxy()
            .contract(self.main_dao().get())
            .franchises()
            .execute_on_dest_context();
        for franchise in franchises.into_iter() {
            if franchise == address {
                return true;
            }
        }

        false
    }

    #[endpoint(test)]
    fn is_franchise_student(
        &self,
        franchise_address: ManagedAddress,
        student_address: ManagedAddress,
    ) -> bool {
        let student: Option<Student<Self::Api>> = if self.blockchain().is_smart_contract(&student_address) {
            self.franchise_dao_contract_proxy()
                .contract(franchise_address)
                .get_student_by_address(student_address)
                .execute_on_dest_context()
        } else {
            self.franchise_dao_contract_proxy()
                .contract(franchise_address)
                .get_student_by_wallet(student_address)
                .execute_on_dest_context()
        };

        student.is_some()
    }

    // proxies
    #[proxy]
    fn dao_contract_proxy(&self) -> tfn_dao::Proxy<Self::Api>;

    #[proxy]
    fn franchise_dao_contract_proxy(&self) -> tfn_franchise_dao::Proxy<Self::Api>;
}
