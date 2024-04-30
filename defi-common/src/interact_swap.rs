//! # Liquidity Swap invocation helper
//!
//! Mini-library for creating interactions with Liquidity Swap Contracts.
//!
//! Assumes that the target contract possesses actions where the shortname and arguments matches
//! the following:
//!
//! ```ignore
//! #[action(shortname=0x01)] deposit(token_address: Address, amount: TokenAmount);
//! #[action(shortname=0x03)] withdraw(token_address: Address, amount: TokenAmount, wait_for_callback: bool);
//! ```

use pbc_contract_common::{
    address::Address,
    events::{EventGroupBuilder, GasCost},
    shortname::Shortname,
};

use crate::{interact_mpc20::MPC20Contract, token_balances::TokenAmount};

/// Represents an individual swap contract on the blockchain.
pub struct SwapContract {
    contract_address: Address,
}

impl SwapContract {
    /// Shortname of the [`SwapContract::deposit`] invocation
    const SHORTNAME_DEPOSIT_SWAP_LOCK: Shortname = Shortname::from_u32(0x01);

    /// Shortname of the [`SwapContract::withdraw`] invocation
    const SHORTNAME_WITHDRAW_SWAP_LOCK: Shortname = Shortname::from_u32(0x03);

    const DEPOSIT_BASE_COST: GasCost = 1770;

    /// Gas amount sufficient for [`SwapContract::deposit`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_DEPOSIT: GasCost =
        Self::DEPOSIT_BASE_COST + MPC20Contract::GAS_COST_TRANSFER;

    const WITHDRAW_BASE_COST: GasCost = 633;

    /// Gas amount sufficient for [`SwapContract::withdraw`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_WITHDRAW: GasCost =
        Self::WITHDRAW_BASE_COST + MPC20Contract::GAS_COST_TRANSFER_FROM;

    /// Create a new swap contract representation at `contract_address`.
    pub fn at_address(contract_address: Address) -> Self {
        Self { contract_address }
    }

    /// Create an interaction with the `self` swap contract, for depositing an `amount` of
    /// `token`s from calling contract into the swap contract.
    /// Requires that the calling contract has [`approve`](crate::interact_mpc20::MPC20Contract::approve)d the swap contract.
    pub fn deposit(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        token: &Address,
        amount: TokenAmount,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_DEPOSIT_SWAP_LOCK)
            .argument(*token)
            .argument(amount)
            .with_cost(Self::GAS_COST_DEPOSIT)
            .done();
    }

    /// Create an interaction with the `self` swap contract, for withdrawing an `amount` of
    /// `token`s from the swap contract.
    ///
    /// This will transfer the tokens held by the swap contract to the sender of the withdraw interaction.
    /// If `wait_for_callback` is true, a callback is added after the internal transfer invocation
    /// to the withdrawn token, ensuring any callbacks added to `event_group_builder` after calling
    /// this function happens after the withdrawal is complete.
    pub fn withdraw(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        token: &Address,
        amount: TokenAmount,
        wait_for_callback: bool,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_WITHDRAW_SWAP_LOCK)
            .argument(*token)
            .argument(amount)
            .argument(wait_for_callback)
            .with_cost(Self::GAS_COST_WITHDRAW)
            .done();
    }
}
