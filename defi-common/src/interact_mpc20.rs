//! # MPC20 invocation helper
//!
//! Mini-library for creating interactions with [MPC20
//! contracts](https://partisiablockchain.gitlab.io/documentation/smart-contracts/integration/mpc-20-token-contract.html),
//! as defined by the [MPC20
//! standard](https://partisiablockchain.gitlab.io/documentation/smart-contracts/integration/mpc-20-token-contract.html)
//! based on the [ERC-20](https://ethereum.org/en/developers/docs/standards/tokens/erc-20/)
//! standard.
//!
//! Assumes that the target contract possesses actions where the shortname and arguments matches
//! the following:
//!
//! ```ignore
//! #[action(shortname=0x01)] transfer(to: Address, amount: u128);
//! #[action(shortname=0x03)] transfer_from(from: Address, to: Address, amount: u128);
//! #[action(shortname=0x05)] approve(spender: Address, amount: u128);
//! ```
//!
//! The root state struct is named TokenState and each of the following state fields exist in the
//! root state struct or a sub-struct that has a 1-1 composition with the root state struct where
//! names and types match exactly:
//!
//! ```ignore
//! balances: Map<Address, u128>
//! name: String
//! symbol: String
//! decimals: u8
//! ```

use pbc_contract_common::{
    address::Address,
    events::{EventGroupBuilder, GasCost},
    shortname::Shortname,
};

use crate::token_balances::TokenAmount;

/// Represents an individual [MPC20 contract](https://partisiablockchain.gitlab.io/documentation/smart-contracts/integration/mpc-20-token-contract.html) on the blockchain.
pub struct MPC20Contract {
    contract_address: Address,
}

/// Token transfer amounts for the token contract.
pub type TokenTransferAmount = u128;

impl MPC20Contract {
    /// Shortname of the [`MPC20Contract::transfer`] invocation
    const SHORTNAME_TRANSFER: Shortname = Shortname::from_u32(0x01);

    /// Shortname of the [`MPC20Contract::transfer_from`] invocation
    const SHORTNAME_TRANSFER_FROM: Shortname = Shortname::from_u32(0x03);

    /// Shortname of the [`MPC20Contract::approve`] invocation
    const SHORTNAME_APPROVE: Shortname = Shortname::from_u32(0x05);

    /// Shortname of the [`MPC20Contract::approve_relative`] invocation
    const SHORTNAME_APPROVE_RELATIVE: Shortname = Shortname::from_u32(0x07);

    /// Gas amount sufficient for [`MPC20Contract::transfer`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_TRANSFER: GasCost = 15500;

    /// Gas amount sufficient for MPC20 [`MPC20Contract::transfer_from`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_TRANSFER_FROM: GasCost = 15500;

    /// Gas amount sufficient for MPC20 [`MPC20Contract::approve`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_APPROVE: GasCost = 3000;

    /// Gas amount sufficient for MPC20 [`MPC20Contract::approve_relative`] invocation.
    ///
    /// Guarantees that the invocation does not fail due to insufficient gas.
    pub const GAS_COST_APPROVE_RELATIVE: GasCost = 1400;

    /// Create new token contract representation for the given `contract_address`.
    ///
    /// It is expected that the given address indicates a [MPC20
    /// contract](https://partisiablockchain.gitlab.io/documentation/smart-contracts/integration/mpc-20-token-contract.html).
    pub fn at_address(contract_address: Address) -> Self {
        Self { contract_address }
    }

    /// Create an interaction with the `self` token contract, for transferring an `amount` of
    /// tokens from calling contract to `receiver`.
    pub fn transfer(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        receiver: &Address,
        amount: TokenTransferAmount,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_TRANSFER)
            .argument(*receiver)
            .argument(amount)
            .with_cost(Self::GAS_COST_TRANSFER)
            .done();
    }

    /// Create an interaction with the `self` token contract, for transferring an `amount` of
    /// tokens from `sender` to `receiver`. Requires that calling contract have been given an
    /// allowance by `sender`, by using [`Self::approve`].
    pub fn transfer_from(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        sender: &Address,
        receiver: &Address,
        amount: TokenTransferAmount,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_TRANSFER_FROM)
            .argument(*sender)
            .argument(*receiver)
            .argument(amount)
            .with_cost(Self::GAS_COST_TRANSFER_FROM)
            .done();
    }

    /// Create an interaction with the `self` token contract, for approving an `approval_amount` of
    /// tokens owned by the sender of the interaction, to be handled by the `approved` contract.
    pub fn approve(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        approved: &Address,
        approval_amount: TokenAmount,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_APPROVE)
            .argument(*approved)
            .argument(approval_amount)
            .with_cost(Self::GAS_COST_APPROVE)
            .done();
    }

    /// Create an interaction with the `self` token contract, for approving an additional `approval_amount` of
    /// tokens owned by the sender of the interaction, to be handled by the `approved` contract.
    ///
    /// Not part of the MPC20 standard, but a useful extension supported by the `token` and
    /// `token-v2` contracts.
    pub fn approve_relative(
        &self,
        event_group_builder: &mut EventGroupBuilder,
        approved: &Address,
        approval_amount: i128,
    ) {
        event_group_builder
            .call(self.contract_address, Self::SHORTNAME_APPROVE_RELATIVE)
            .argument(*approved)
            .argument(approval_amount)
            .with_cost(Self::GAS_COST_APPROVE_RELATIVE)
            .done();
    }
}
