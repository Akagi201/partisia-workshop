#![doc = include_str!("../README.md")]

use std::ops::{Add, Sub};

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_codegen::*;
use pbc_contract_common::{address::Address, avl_tree_map::AvlTreeMap, context::ContractContext};
use pbc_traits::ReadWriteState;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

/// Custom struct for the state of the contract.
///
/// The "state" attribute is attached.
///
/// ### Fields:
///
/// * `name`: [`String`], the name of the token - e.g. "MyToken".\
///
/// * `symbol`: [`String`], the symbol of the token. E.g. "HIX".\
///
/// * `decimals`: [`u8`], the number of decimals the token uses - e.g. 8,
/// means to divide the token amount by `100000000` to get its user representation.\
///
/// * `owner`: [`Address`], owner of the contract.
///
/// * `total_supply`: [`u128`], current amount of tokens for the TokenContract.
///
/// * `balances`: [`AvlTreeMap<Address, u128>`], ledger for the accounts associated with the contract.
///
/// * `allowed`: [`AvlTreeMap<AllowedAddress, u128>`], allowance from an owner to a spender.
#[state]
pub struct TokenState {
    name: String,
    decimals: u8,
    symbol: String,
    owner: Address,
    total_supply: u128,
    balances: AvlTreeMap<Address, u128>,
    allowed: AvlTreeMap<AllowedAddress, u128>,
}

/// Address pair representing some allowance. Owner allows spender to spend an amount of tokens.
///
/// ### Fields:
///
/// * `owner`: [`Address`], owner of tokens.
///
/// * `spender`: [`Address`], spender of tokens.
#[derive(ReadWriteState, CreateTypeSpec, Eq, Ord, PartialEq, PartialOrd)]
pub struct AllowedAddress {
    owner: Address,
    spender: Address,
}

/// Extension trait for inserting into a map holding balances.
/// In a balance map only non-zero values are stored.
/// If a key has no value in the map the implied value is zero.
trait BalanceMap<K: Ord, V> {
    /// Insert into the map if `value` is not zero.
    /// Removes the key from the map if `value` is zero.
    ///
    /// ## Arguments
    ///
    /// * `key`: Key for map.
    ///
    /// * `value`: The balance value to insert.
    fn insert_balance(&mut self, key: K, value: V);
}

/// Extension for [`AvlTreeMap`] allowing the use of [`BalanceMap::insert_balance`].
///
/// This implementation defines zero as `forall v: v - v = 0` (the subtract of a value from itself), to support a large variety
/// of values. Might not work correctly for unusual implementations of [`Sub::sub`].
impl<V: Sub<V, Output = V> + PartialEq + Copy + ReadWriteState, K: ReadWriteState + Ord>
    BalanceMap<K, V> for AvlTreeMap<K, V>
{
    #[allow(clippy::eq_op)]
    fn insert_balance(&mut self, key: K, value: V) {
        let zero = value - value;
        if value == zero {
            self.remove(&key);
        } else {
            self.insert(key, value);
        }
    }
}

impl TokenState {
    /// Gets the balance of the specified address.
    ///
    /// ### Parameters:
    ///
    /// * `owner`: The [`Address`] to query the balance of.
    ///
    /// ### Returns:
    ///
    /// An [`u64`] representing the amount owned by the passed address.
    pub fn balance_of(&self, owner: &Address) -> u128 {
        self.balances.get(owner).unwrap_or(0)
    }

    /// Function to check the amount of tokens that an owner allowed to a spender.
    ///
    /// ### Parameters:
    ///
    /// * `owner`: [`Address`] The address which owns the funds.
    ///
    /// * `spender`: [`Address`] The address which will spend the funds.
    ///
    /// ### Returns:
    ///
    /// A [`u64`] specifying the amount whicher `spender` is still allowed to withdraw from `owner`.
    pub fn allowance(&self, owner: &Address, spender: &Address) -> u128 {
        self.allowed
            .get(&AllowedAddress {
                owner: *owner,
                spender: *spender,
            })
            .unwrap_or(0)
    }

    /// Updates the internal allowance map, overwriting `owner`'s allowance for `spender` to `amount`.
    ///
    /// If `owner` does not currently have any allowance, a new entry is added to `self`.
    fn update_allowance(&mut self, owner: Address, spender: Address, amount: u128) {
        self.allowed
            .insert_balance(AllowedAddress { owner, spender }, amount);
    }

    /// Updates the internal allowance map, adding `delta` allowance for `spender` to additionally
    /// spend of on behalf of `owner`.
    ///
    /// If `owner` does not currently have any allowance, a new entry is added to `self`, with `delta`
    /// as the initial amount.
    /// If `delta` is negative, the allowance is lowered.
    /// Panics if adding `delta` would overflow, or the allowed balance would become negative.
    fn update_allowance_relative(&mut self, owner: Address, spender: Address, delta: i128) {
        let existing_allowance = self.allowance(&owner, &spender);
        let new_allowance = existing_allowance
            .checked_add_signed(delta)
            .expect("Allowance would become negative.");
        self.update_allowance(owner, spender, new_allowance);
    }
}

/// Initial function to bootstrap the contracts state. Must return the state-struct.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], initial context.
///
/// * `name`: [`String`], the name of the token - e.g. "MyToken".\
///
/// * `symbol`: [`String`], the symbol of the token. E.g. "HIX".\
///
/// * `decimals`: [`u8`], the number of decimals the token uses - e.g. 8,
/// means to divide the token amount by `100000000` to get its user representation.\
///
/// * `total_supply`: [`u128`], current amount of tokens for the TokenContract.
///
/// ### Returns:
///
/// The new state object of type [`TokenState`] with an initialized ledger.
#[init]
pub fn initialize(
    ctx: ContractContext,
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
) -> TokenState {
    let mut balances = AvlTreeMap::new();
    balances.insert_balance(ctx.sender, total_supply);

    TokenState {
        name,
        symbol,
        decimals,
        owner: ctx.sender,
        total_supply,
        balances,
        allowed: AvlTreeMap::new(),
    }
}

/// Represents the type of a transfer.
#[derive(ReadWriteRPC, CreateTypeSpec)]
pub struct Transfer {
    /// The address to transfer to.
    pub to: Address,
    /// The amount to transfer.
    pub amount: u128,
}

/// Transfers `amount` of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `amount`: [`u128`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
#[action(shortname = 0x01)]
pub fn transfer(
    context: ContractContext,
    state: TokenState,
    to: Address,
    amount: u128,
) -> TokenState {
    core_transfer(context.sender, state, to, amount)
}

/// Transfers a bulk of `amount` of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `transfers`: [`Vec[Transfer]`], vector of [the address to transfer to, amount to transfer].
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
#[action(shortname = 0x02)]
pub fn bulk_transfer(
    context: ContractContext,
    mut state: TokenState,
    transfers: Vec<Transfer>,
) -> TokenState {
    for t in transfers {
        state = core_transfer(context.sender, state, t.to, t.amount);
    }
    state
}

/// Transfers `amount` of tokens from address `from` to address `to`.\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `amount`: [`u128`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
#[action(shortname = 0x03)]
pub fn transfer_from(
    context: ContractContext,
    state: TokenState,
    from: Address,
    to: Address,
    amount: u128,
) -> TokenState {
    core_transfer_from(context.sender, state, from, to, amount)
}

/// Transfers a bulk of `amount` of tokens to address `to` from address `from` .\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `transfers`: [`Vec[Transfer]`], vector of [the address to transfer to, amount to transfer].
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
#[action(shortname = 0x04)]
pub fn bulk_transfer_from(
    context: ContractContext,
    mut state: TokenState,
    from: Address,
    transfers: Vec<Transfer>,
) -> TokenState {
    for t in transfers {
        state = core_transfer_from(context.sender, state, from, t.to, t.amount);
    }
    state
}

/// Allows `spender` to withdraw from the owners account multiple times, up to the `amount`.
/// If this function is called again it overwrites the current allowance with `amount`.
///
/// ### Parameters:
///
/// * `context`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `spender`: [`Address`], the address of the spender.
///
/// * `amount`: [`u128`], approved amount.
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
#[action(shortname = 0x05)]
pub fn approve(
    context: ContractContext,
    mut state: TokenState,
    spender: Address,
    amount: u128,
) -> TokenState {
    state.update_allowance(context.sender, spender, amount);
    state
}

/// Allows `spender` to withdraw `delta` additional tokens from the owners account, relative to any
/// pre-existing allowance.
///
/// If there is no pre-existing allowance, this is equivalent to [`approve`], with `delta` = `amount`.
/// If `delta` is negative, the allowance is lowered.
/// If the resulting allowance is negative, the call fails, and the allowance is unchanged.
#[action(shortname = 0x07)]
pub fn approve_relative(
    context: ContractContext,
    mut state: TokenState,
    spender: Address,
    delta: i128,
) -> TokenState {
    state.update_allowance_relative(context.sender, spender, delta);
    state
}

/// Transfers `amount` of tokens to address `to` from the caller.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend.
/// If the sender's account goes to 0, the sender's address is removed from state.
///
/// ### Parameters:
///
/// * `sender`: [`Address`], the sender of the transaction.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `amount`: [`u128`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
pub fn core_transfer(
    sender: Address,
    mut state: TokenState,
    to: Address,
    amount: u128,
) -> TokenState {
    let from_amount = state.balance_of(&sender);
    let o_new_from_amount = from_amount.checked_sub(amount);
    match o_new_from_amount {
        Some(new_from_amount) => {
            state.balances.insert_balance(sender, new_from_amount);
        }
        None => {
            panic!(
                "Insufficient funds for transfer: {}/{}",
                from_amount, amount
            );
        }
    }
    let to_amount = state.balance_of(&to);
    state.balances.insert_balance(to, to_amount.add(amount));
    state
}

/// Transfers `amount` of tokens from address `from` to address `to`.\
/// This requires that the sender is allowed to do the transfer by the `from`
/// account through the `approve` action.
/// The function throws if the message caller's account
/// balance does not have enough tokens to spend, or if the tokens were not approved.
///
/// ### Parameters:
///
/// * `sender`: [`Address`], the sender of the transaction.
///
/// * `state`: [`TokenState`], the current state of the contract.
///
/// * `from`: [`Address`], the address to transfer from.
///
/// * `to`: [`Address`], the address to transfer to.
///
/// * `amount`: [`u128`], amount to transfer.
///
/// ### Returns
///
/// The new state object of type [`TokenState`] with an updated ledger.
pub fn core_transfer_from(
    sender: Address,
    mut state: TokenState,
    from: Address,
    to: Address,
    amount: u128,
) -> TokenState {
    let from_allowed = state.allowance(&from, &sender);
    let o_new_allowed_amount = from_allowed.checked_sub(amount);
    match o_new_allowed_amount {
        Some(new_allowed_amount) => {
            state.update_allowance(from, sender, new_allowed_amount);
        }
        None => {
            panic!(
                "Insufficient allowance for transfer_from: {}/{}",
                from_allowed, amount
            );
        }
    }
    core_transfer(from, state, to, amount)
}
