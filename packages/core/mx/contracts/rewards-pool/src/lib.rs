#![no_std]

pub mod constants;

use constants::{Rewards, Reward};
multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait RewardsPoolContract {

    #[init]
    fn init(
        &self,
        rewards_token: TokenIdentifier,
        protocol_fee: BigUint,
        staking_contract_address: ManagedAddress,
    ) {
        self.rewards_token().set_if_empty(rewards_token);
        self.protocol_fee().set_if_empty(protocol_fee);
        self.staking_contract_address().set_if_empty(staking_contract_address);
    }

    /// Add rewards record
    /// Protocol fee is deduced from the rewards amount
    #[payable("*")]
    #[endpoint(addReward)]
    fn add_reward(&self, escrow_address: ManagedAddress, slasher: ManagedAddress) {
        self.require_only_staking();

        let payment = self.call_value().single_esdt();
        require!(payment.token_identifier == self.rewards_token().get(), "Invalid token");
        require!(payment.token_nonce == 0, "Invalid token nonce");
        require!(payment.amount > 0, "Amount must be greater than 0");

        let tokens = payment.amount;
        let protocol_fee = self.protocol_fee().get();
        if tokens < protocol_fee {
            self.total_fee().update(|total_fee| *total_fee += tokens);
            return
        }

        let rewards_after_fee = tokens - &protocol_fee;
        if rewards_after_fee > 0 {
            let new_reward_entry = Reward {
                escrow_address: escrow_address.clone(),
                slasher: slasher.clone(),
                tokens: rewards_after_fee.clone(),
            };
            self.rewards(&escrow_address).update(|rewards| rewards.push(new_reward_entry));
        }

        self.total_fee().update(|total_fee| *total_fee += &protocol_fee);
        self.rewards_added_event(escrow_address, slasher, rewards_after_fee);
    }

    /// Distribute rewards for allocation
    #[endpoint(distributeRewards)]
    fn distribute_rewards(&self, escrow_address: ManagedAddress) {
        let rewards_for_escrow = self.rewards(&escrow_address).get();
        for reward in rewards_for_escrow.iter() {
            self.send().direct_esdt(&reward.slasher, &self.rewards_token().get(), 0, &reward.tokens);
        }
        self.rewards(&escrow_address).clear();
    }

    #[only_owner]
    #[endpoint(withdraw)]
    fn withdraw(&self) {
        let total_fee = self.total_fee().get();
        let rewards_token = self.rewards_token().get();
        let caller = self.blockchain().get_caller();
        self.send().direct_esdt(&caller, &rewards_token, 0, &total_fee);
        self.total_fee().clear();
    }

    fn require_only_staking(&self) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.staking_contract_address().get(), "Caller is not staking contract");
    }

    #[view(getRewards)]
    fn get_rewards(&self, escrow_address: ManagedAddress) -> MultiValueEncoded<Reward<Self::Api>> {
        let rewards = self.rewards(&escrow_address).get();
        MultiValueEncoded::from(rewards)
    }

    #[view(getRewardsToken)]
    #[storage_mapper("rewards_token")]
    fn rewards_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getProtocolFee)]
    #[storage_mapper("protocol_fee")]
    fn protocol_fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getTotalFee)]
    #[storage_mapper("total_fee")]
    fn total_fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewards")]
    fn rewards(&self, escrow_address: &ManagedAddress) -> SingleValueMapper<Rewards<Self::Api>>;

    #[view(getStakingContractAddress)]
    #[storage_mapper("staking_contract_address")]
    fn staking_contract_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[event("rewards_added")]
    fn rewards_added_event(
        &self,
        #[indexed] escrow_address: ManagedAddress,
        #[indexed] slasher: ManagedAddress,
        tokens: BigUint
    );
}