use crate::*;
use cosmwasm_std::{ensure, Addr, Deps, DepsMut, Event, Response, StdResult};
use error::ContractError;
use state::{UserStatus, OWNER, WHITELIST_AGENTS, WHITELIST_USERS};

/// Adds an agent to the whitelist.
///
/// Any existing agent may add another agent (the caller must already be an agent). Agent management
/// is a role distinct from the contract owner. See "Whitelist" in the README.
pub fn add_agent(
    deps: DepsMut,
    caller: Addr,
    new_agent: &String,
) -> Result<Response, ContractError> {
    // check that the caller is an agent
    check_agent(deps.as_ref(), &caller)?;

    // validate agent address
    let new_agent_addr = deps.api.addr_validate(new_agent)?;

    // check that the new agent is not the owner
    let owner = OWNER.load(deps.storage)?;
    ensure!(new_agent_addr != owner, ContractError::OwnerCannotBeAdded);

    // check that the new agent is already an agent
    ensure!(
        !WHITELIST_AGENTS.has(deps.storage, &new_agent_addr),
        ContractError::AgentAlreadyExists
    );

    // add the new agent
    WHITELIST_AGENTS.save(deps.storage, &new_agent_addr, &())?;

    // emit the event
    Ok(Response::new().add_event(Event::new("agent_added").add_attribute("new_agent", new_agent)))
}

/// Removes an agent from the whitelist.
pub fn remove_agent(
    deps: DepsMut,
    caller: Addr,
    agent_to_remove: &String,
) -> Result<Response, ContractError> {
    // check that the caller is an agent
    check_agent(deps.as_ref(), &caller)?;

    let agent_to_remove_addr = deps.api.addr_validate(agent_to_remove)?;

    // check that the agent to remove is not the owner
    let owner = OWNER.load(deps.storage)?;
    ensure!(
        agent_to_remove_addr != owner,
        ContractError::OwnerCannotBeRemoved
    );

    // check that the agent to remove is an agent
    ensure!(
        WHITELIST_AGENTS.has(deps.storage, &agent_to_remove_addr),
        ContractError::AgentDoesNotExist
    );

    // remove the agent
    WHITELIST_AGENTS.remove(deps.storage, &agent_to_remove_addr);

    // emit the event
    Ok(Response::new()
        .add_event(Event::new("agent_removed").add_attribute("removed_agent", agent_to_remove)))
}

/// Checks whether an address is an agent or the owner.
/// Returns CallerIsNotAgent error if it is neither.
fn check_agent(deps: Deps, agent: &Addr) -> Result<(), ContractError> {
    ensure!(is_agent(deps, agent)?, ContractError::CallerIsNotAgent);
    Ok(())
}

/// Checks whether a user is whitelisted.
/// Returns UserNotWhitelisted error if not.
pub(crate) fn check_whitelisted(deps: Deps, user: &Addr) -> Result<(), ContractError> {
    ensure!(
        is_user_whitelisted(deps, user),
        ContractError::UserNotWhitelisted
    );
    Ok(())
}

/// Checks whether an address is an agent or the owner.
/// Returns true if it is either, false otherwise.
pub fn is_agent(deps: Deps, agent: &Addr) -> StdResult<bool> {
    let owner = OWNER.load(deps.storage)?;

    Ok(owner == agent || WHITELIST_AGENTS.has(deps.storage, agent))
}

/// Adds a user to the whitelist.
pub fn add_user_to_whitelist(
    deps: DepsMut,
    caller: Addr,
    user: &String,
) -> Result<Response, ContractError> {
    // check if the caller is an agent
    check_agent(deps.as_ref(), &caller)?;

    // validate user address
    let user_addr = deps.api.addr_validate(user.as_str())?;

    // check if the user is already whitelisted
    let current_user_status = get_current_user_status(deps.as_ref(), &user_addr)?;
    ensure!(
        current_user_status != UserStatus::Whitelisted,
        ContractError::UserAlreadyWhitelisted
    );

    // add the user to the whitelist
    WHITELIST_USERS.save(deps.storage, &user_addr, &UserStatus::Whitelisted)?;

    // emit the event
    Ok(Response::new().add_event(
        Event::new("whitelisting_status_changed")
            .add_attribute("user", user)
            .add_attribute("old_status", current_user_status.to_string())
            .add_attribute("new_status", UserStatus::Whitelisted.to_string()),
    ))
}

/// Adds a user to the blacklist.
pub fn add_user_to_blacklist(
    deps: DepsMut,
    caller: Addr,
    user: &String,
) -> Result<Response, ContractError> {
    // check if the caller is an agent
    check_agent(deps.as_ref(), &caller)?;

    // validate user address
    let user_addr = deps.api.addr_validate(user.as_str())?;

    // check if the user is already blacklisted
    let current_user_status = get_current_user_status(deps.as_ref(), &user_addr).unwrap();
    ensure!(
        current_user_status != UserStatus::Blacklisted,
        ContractError::UserAlreadyBlacklisted
    );

    // add the user to the blacklist
    WHITELIST_USERS.save(deps.storage, &user_addr, &UserStatus::Blacklisted)?;

    // emit the event
    Ok(Response::new().add_event(
        Event::new("whitelisting_status_changed")
            .add_attribute("user", user)
            .add_attribute("old_status", current_user_status.to_string())
            .add_attribute("new_status", UserStatus::Blacklisted.to_string()),
    ))
}

/// Removes a user's status.
pub fn clear_user_status(
    deps: DepsMut,
    caller: Addr,
    user: &String,
) -> Result<Response, ContractError> {
    // check if the caller is an agent
    check_agent(deps.as_ref(), &caller)?;

    let user_addr = deps.api.addr_validate(user.as_str())?;

    // check if the user has a status
    let current_user_status = get_current_user_status(deps.as_ref(), &user_addr)?;
    ensure!(
        current_user_status != UserStatus::NoStatus,
        ContractError::UserStatusAlreadyCleared
    );

    // clear the user status
    WHITELIST_USERS.remove(deps.storage, &user_addr);

    // emit the event
    Ok(Response::new().add_event(
        Event::new("whitelisting_status_changed")
            .add_attribute("user", user)
            .add_attribute("old_status", current_user_status.to_string())
            .add_attribute("new_status", UserStatus::NoStatus.to_string()),
    ))
}

/// Gets the current whitelist status of a user.
pub fn get_current_user_status(deps: Deps, user: &Addr) -> StdResult<UserStatus> {
    WHITELIST_USERS
        .may_load(deps.storage, user)?
        .map_or_else(|| Ok(UserStatus::NoStatus), Ok)
}

/// Checks if a user is whitelisted.
pub fn is_user_whitelisted(deps: Deps, user: &Addr) -> bool {
    get_current_user_status(deps, user).unwrap() == UserStatus::Whitelisted
}

/// Checks if a user is blacklisted.
pub fn is_user_blacklisted(deps: Deps, user: &Addr) -> bool {
    get_current_user_status(deps, user).unwrap() == UserStatus::Blacklisted
}
