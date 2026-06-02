#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// A team member or pending invitee.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamUser {
    /// User identifier.
    pub id: i64,
    /// Display name.
    pub full_name: Option<String>,
    /// Email address.
    pub email: String,
    /// Team role (e.g. `admin`, `viewer`, `billing`).
    pub role: Option<String>,
    /// Membership status (e.g. `active`, `pending`).
    pub status: Option<String>,
    /// When the user was added.
    pub created_at: Option<String>,
    /// Profile photo URL.
    pub photo_url: Option<String>,
    /// Whether this user is the primary user on the account.
    pub account_primary_user: Option<bool>,
}

/// Summary representation of a team in list responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSummary {
    /// Team identifier.
    pub id: i64,
    /// Team name.
    pub name: String,
    /// Current member count.
    pub members_count: Option<i64>,
    /// Active team members.
    #[serde(default)]
    pub users: Vec<TeamUser>,
}

/// Full team detail including pending invites.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDetail {
    /// Team identifier.
    pub id: i64,
    /// Team name.
    pub name: String,
    /// Default role assigned to newly invited members.
    pub default_role: Option<String>,
    /// Current member count.
    pub members_count: Option<i64>,
    /// Active team members.
    #[serde(default)]
    pub users: Vec<TeamUser>,
    /// Invites that have not yet been accepted.
    #[serde(default)]
    pub pending_invites: Vec<TeamUser>,
}

/// Response from `list_teams`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTeamsResponse {
    /// Teams on the account.
    #[serde(default)]
    pub data: Vec<TeamSummary>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `create_team`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTeamRequest {
    /// Team name.
    pub name: String,
}

/// Inner data for `create_team` responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamData {
    /// Team identifier.
    pub id: i64,
    /// Team name.
    pub name: String,
    /// Default role for newly invited members.
    pub default_role: Option<String>,
    /// Initial member count.
    pub members_count: Option<i64>,
}

/// Response from `create_team`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamResponse {
    /// The newly created team.
    pub data: Option<CreateTeamData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Response from `get_team`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTeamResponse {
    /// The team's full detail.
    pub data: Option<TeamDetail>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Inner data for `delete_team` responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTeamData {
    /// Human-readable confirmation message.
    pub message: Option<String>,
}

/// Response from `delete_team`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteTeamResponse {
    /// Deletion result payload.
    pub data: Option<DeleteTeamData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// A team's endpoint association.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEndpoint {
    /// Endpoint identifier.
    pub id: i64,
    /// Endpoint subdomain.
    pub subdomain: String,
    /// Blockchain the endpoint serves.
    pub chain: Option<String>,
    /// Network within the chain.
    pub network: Option<String>,
}

/// Response from `list_team_endpoints`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTeamEndpointsResponse {
    /// Endpoints accessible to the team.
    #[serde(default)]
    pub data: Vec<TeamEndpoint>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `update_team_endpoints`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateTeamEndpointsRequest {
    /// Endpoint ids to associate with the team; pass an empty array to remove all.
    pub endpoint_ids: Vec<String>,
}

/// Inner data for `update_team_endpoints` responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamEndpointsData {
    /// `true` when the association update succeeded.
    pub success: Option<bool>,
}

/// Response from `update_team_endpoints`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTeamEndpointsResponse {
    /// Update result.
    pub data: Option<UpdateTeamEndpointsData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `invite_team_member`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct InviteTeamMemberRequest {
    /// Email address to invite.
    pub email: String,
    /// Full name (required for new users).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    /// Team role (`admin`, `viewer`, or `billing`); required for new users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Response from `invite_team_member`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteTeamMemberResponse {
    /// The invited user and their invitation status.
    pub data: Option<TeamUser>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `remove_team_member`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct RemoveTeamMemberRequest {
    /// When true, also delete the user entirely rather than just removing them from the team.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destroy_user: Option<bool>,
}

/// Shared message-shaped data wrapper for team operations.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessageData {
    /// Human-readable confirmation message.
    pub message: Option<String>,
}

/// Response from `remove_team_member`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveTeamMemberResponse {
    /// Operation result message.
    pub data: Option<TeamMessageData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Response from `resend_team_invite`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendTeamInviteResponse {
    /// Operation result message.
    pub data: Option<TeamMessageData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

// ── Python conveniences (__repr__, to_dict) ───────────────────────────────
// Generated by the `python_repr_dict!` macro (see crates/core/src/python_macros.rs).

#[cfg(feature = "python")]
mod python_repr_impls {
    use super::*;
    crate::python_repr_dict!(TeamUser);
    crate::python_repr_dict!(TeamSummary);
    crate::python_repr_dict!(TeamDetail);
    crate::python_repr_dict!(ListTeamsResponse);
    crate::python_repr_dict!(CreateTeamRequest);
    crate::python_repr_dict!(CreateTeamData);
    crate::python_repr_dict!(CreateTeamResponse);
    crate::python_repr_dict!(GetTeamResponse);
    crate::python_repr_dict!(DeleteTeamData);
    crate::python_repr_dict!(DeleteTeamResponse);
    crate::python_repr_dict!(TeamEndpoint);
    crate::python_repr_dict!(ListTeamEndpointsResponse);
    crate::python_repr_dict!(UpdateTeamEndpointsRequest);
    crate::python_repr_dict!(UpdateTeamEndpointsData);
    crate::python_repr_dict!(UpdateTeamEndpointsResponse);
    crate::python_repr_dict!(InviteTeamMemberRequest);
    crate::python_repr_dict!(InviteTeamMemberResponse);
    crate::python_repr_dict!(RemoveTeamMemberRequest);
    crate::python_repr_dict!(TeamMessageData);
    crate::python_repr_dict!(RemoveTeamMemberResponse);
    crate::python_repr_dict!(ResendTeamInviteResponse);
}
