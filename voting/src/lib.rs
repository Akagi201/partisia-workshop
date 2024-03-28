#![doc = include_str!("../README.md")]
#![allow(unused_variables)]

use pbc_contract_codegen::*;
use pbc_contract_common::{
    address::Address,
    context::ContractContext,
    sorted_vec_map::{SortedVecMap, SortedVecSet},
};

/// The state of the vote, which is persisted on-chain.
#[state]
pub struct VoteState {
    /// Identification of the proposal being voted for.
    pub proposal_id: u64,
    /// The list of eligible voters.
    pub voters: SortedVecSet<Address>,
    /// The deadline of the vote in UTC millis
    /// (milliseconds after 1970-01-01 00:00:00 UTC)
    pub deadline_utc_millis: i64,
    /// The votes cast by the voters.
    /// true is for the proposal, false is against.
    pub votes: SortedVecMap<Address, bool>,
    /// The result of the vote.
    /// None until the votes has been counted,
    /// Some(true) if the proposal passed,
    /// Some(false) if the proposal failed.
    pub result: Option<bool>,
}

/// Initialize a new vote for a proposal
///
/// # Arguments
///
/// * `_ctx` - the contract context containing information about the sender and the blockchain.
/// * `proposal_id` - the id of the proposal.
/// * `voters` - the list of eligible voters.
/// * `deadline_utc_millis` - deadline of the vote in UTC millis.
///
/// # Returns
///
/// The initial state of the vote.
#[init]
pub fn initialize(
    _ctx: ContractContext,
    proposal_id: u64,
    voters: Vec<Address>,
    deadline_utc_millis: i64,
) -> VoteState {
    assert_ne!(voters.len(), 0, "Voters are required");
    let unique_voters: SortedVecSet<Address> = voters.iter().cloned().collect();
    assert_eq!(
        voters.len(),
        unique_voters.len(),
        "All voters must be unique"
    );
    VoteState {
        proposal_id,
        voters: unique_voters,
        deadline_utc_millis,
        votes: SortedVecMap::new(),
        result: None,
    }
}

/// Cast a vote for the proposal.
/// The vote is cast by the sender of the action.
/// Voters can cast and update their vote until the deadline.
///
/// # Arguments
///
/// * `ctx` - the contract context containing information about the sender and the blockchain.
/// * `state` - the current state of the vote.
/// * `vote` - the vote being cast by the sender.
///
/// # Returns
///
/// The updated vote state reflecting the newly cast vote.
#[action(shortname = 0x01)]
pub fn vote(ctx: ContractContext, mut state: VoteState, vote: bool) -> VoteState {
    assert!(
        state.result.is_none() && ctx.block_production_time < state.deadline_utc_millis,
        "The deadline has passed"
    );
    assert!(state.voters.contains(&ctx.sender), "Not an eligible voter");
    state.votes.insert(ctx.sender, vote);
    state
}

/// Count the votes and publish the result.
/// Counting will fail if the deadline has not passed.
///
/// # Arguments
///
/// * `ctx` - the contract context containing information about the sender and blockchain.
/// * `state` - the current state of the vote.
///
/// # Returns
///
/// The updated state reflecting the result of the vote.
#[action(shortname = 0x02)]
pub fn count(ctx: ContractContext, mut state: VoteState) -> VoteState {
    assert_eq!(state.result, None, "The votes have already been counted");
    assert!(
        ctx.block_production_time >= state.deadline_utc_millis,
        "The deadline has not yet passed"
    );
    let voters_approving = state.votes.values().filter(|vote| **vote).count();
    let vote_passed = voters_approving > state.voters.len() / 2;
    state.result = Some(vote_passed);
    state
}
