multiversx_sc::imports!();

#[multiversx_sc::proxy]
pub trait MainDAOProxy {
    #[payable("*")]
    #[endpoint(addFunds)]
    fn add_funds(&self) {}

    #[view(getGovernanceToken)]
    #[storage_mapper("governance_token")]
    fn governance_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(isFranchise)]
    fn is_franchise(&self, address: ManagedAddress) -> bool;

    #[view(getLaunchpadAddress)]
    #[storage_mapper("launchpad_sc")]
    fn launchpad_sc(&self) -> SingleValueMapper<ManagedAddress>;
}
