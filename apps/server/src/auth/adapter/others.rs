use crate::auth::adapter::SqliteAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{
    ApiKey, ApiKeyOps, AuthError, AuthResult, CreateApiKey, CreateInvitation, CreateMember,
    CreateOrganization, CreatePasskey, CreateTwoFactor, Invitation, InvitationOps, InvitationStatus,
    MemberOps, OrganizationOps, Passkey, PasskeyOps, TwoFactor, TwoFactorOps, UpdateApiKey,
    UpdateOrganization,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DummyOrganization {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub logo: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
impl better_auth::AuthOrganization for DummyOrganization {
    fn id(&self) -> &str { &self.id }
    fn name(&self) -> &str { &self.name }
    fn slug(&self) -> &str { &self.slug }
    fn logo(&self) -> Option<&str> { self.logo.as_deref() }
    fn metadata(&self) -> Option<&serde_json::Value> { self.metadata.as_ref() }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.created_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DummyMember {
    pub id: String,
    pub organization_id: String,
    pub user_id: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
impl better_auth::AuthMember for DummyMember {
    fn id(&self) -> &str { &self.id }
    fn organization_id(&self) -> &str { &self.organization_id }
    fn user_id(&self) -> &str { &self.user_id }
    fn role(&self) -> &str { &self.role }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
}

#[async_trait]
impl OrganizationOps for SqliteAdapter {
    type Organization = DummyOrganization;
    async fn create_organization(&self, _org: CreateOrganization) -> AuthResult<Self::Organization> {
        Err(AuthError::NotImplemented("OrganizationOps".to_string()))
    }
    async fn get_organization_by_id(&self, _id: &str) -> AuthResult<Option<Self::Organization>> { Ok(None) }
    async fn get_organization_by_slug(&self, _slug: &str) -> AuthResult<Option<Self::Organization>> { Ok(None) }
    async fn update_organization(&self, _id: &str, _update: UpdateOrganization) -> AuthResult<Self::Organization> {
        Err(AuthError::NotImplemented("OrganizationOps".to_string()))
    }
    async fn delete_organization(&self, _id: &str) -> AuthResult<()> { Ok(()) }
    async fn list_user_organizations(&self, _user_id: &str) -> AuthResult<Vec<Self::Organization>> { Ok(vec![]) }
}

#[async_trait]
impl MemberOps for SqliteAdapter {
    type Member = DummyMember;
    async fn create_member(&self, _member: CreateMember) -> AuthResult<Self::Member> {
        Err(AuthError::NotImplemented("MemberOps".to_string()))
    }
    async fn get_member(&self, _org_id: &str, _user_id: &str) -> AuthResult<Option<Self::Member>> { Ok(None) }
    async fn get_member_by_id(&self, _id: &str) -> AuthResult<Option<Self::Member>> { Ok(None) }
    async fn update_member_role(&self, _member_id: &str, _role: &str) -> AuthResult<Self::Member> {
        Err(AuthError::NotImplemented("MemberOps".to_string()))
    }
    async fn delete_member(&self, _member_id: &str) -> AuthResult<()> { Ok(()) }
    async fn list_organization_members(&self, _org_id: &str) -> AuthResult<Vec<Self::Member>> { Ok(vec![]) }
    async fn count_organization_members(&self, _org_id: &str) -> AuthResult<usize> { Ok(0) }
    async fn count_organization_owners(&self, _org_id: &str) -> AuthResult<usize> { Ok(0) }
}

#[async_trait]
impl InvitationOps for SqliteAdapter {
    type Invitation = Invitation;
    async fn create_invitation(&self, _invitation: CreateInvitation) -> AuthResult<Self::Invitation> {
        Err(AuthError::NotImplemented("InvitationOps".to_string()))
    }
    async fn get_invitation_by_id(&self, _id: &str) -> AuthResult<Option<Self::Invitation>> { Ok(None) }
    async fn get_pending_invitation(&self, _org_id: &str, _email: &str) -> AuthResult<Option<Self::Invitation>> { Ok(None) }
    async fn update_invitation_status(&self, _id: &str, _status: InvitationStatus) -> AuthResult<Self::Invitation> {
        Err(AuthError::NotImplemented("InvitationOps".to_string()))
    }
    async fn list_organization_invitations(&self, _org_id: &str) -> AuthResult<Vec<Self::Invitation>> { Ok(vec![]) }
    async fn list_user_invitations(&self, _email: &str) -> AuthResult<Vec<Self::Invitation>> { Ok(vec![]) }
}

#[async_trait]
impl TwoFactorOps for SqliteAdapter {
    type TwoFactor = TwoFactor;
    async fn create_two_factor(&self, _two_factor: CreateTwoFactor) -> AuthResult<Self::TwoFactor> {
        Err(AuthError::NotImplemented("TwoFactorOps".to_string()))
    }
    async fn get_two_factor_by_user_id(&self, _user_id: &str) -> AuthResult<Option<Self::TwoFactor>> { Ok(None) }
    async fn update_two_factor_backup_codes(&self, _user_id: &str, _backup_codes: &str) -> AuthResult<Self::TwoFactor> {
        Err(AuthError::NotImplemented("TwoFactorOps".to_string()))
    }
    async fn delete_two_factor(&self, _user_id: &str) -> AuthResult<()> { Ok(()) }
}

#[async_trait]
impl ApiKeyOps for SqliteAdapter {
    type ApiKey = ApiKey;
    async fn create_api_key(&self, _input: CreateApiKey) -> AuthResult<Self::ApiKey> {
        Err(AuthError::NotImplemented("ApiKeyOps".to_string()))
    }
    async fn get_api_key_by_id(&self, _id: &str) -> AuthResult<Option<Self::ApiKey>> { Ok(None) }
    async fn get_api_key_by_hash(&self, _hash: &str) -> AuthResult<Option<Self::ApiKey>> { Ok(None) }
    async fn list_api_keys_by_user(&self, _user_id: &str) -> AuthResult<Vec<Self::ApiKey>> { Ok(vec![]) }
    async fn update_api_key(&self, _id: &str, _update: UpdateApiKey) -> AuthResult<Self::ApiKey> {
        Err(AuthError::NotImplemented("ApiKeyOps".to_string()))
    }
    async fn delete_api_key(&self, _id: &str) -> AuthResult<()> { Ok(()) }
    async fn delete_expired_api_keys(&self) -> AuthResult<usize> { Ok(0) }
}

#[async_trait]
impl PasskeyOps for SqliteAdapter {
    type Passkey = Passkey;
    async fn create_passkey(&self, _input: CreatePasskey) -> AuthResult<Self::Passkey> {
        Err(AuthError::NotImplemented("PasskeyOps".to_string()))
    }
    async fn get_passkey_by_id(&self, _id: &str) -> AuthResult<Option<Self::Passkey>> { Ok(None) }
    async fn get_passkey_by_credential_id(&self, _credential_id: &str) -> AuthResult<Option<Self::Passkey>> { Ok(None) }
    async fn list_passkeys_by_user(&self, _user_id: &str) -> AuthResult<Vec<Self::Passkey>> { Ok(vec![]) }
    async fn update_passkey_counter(&self, _id: &str, _counter: u64) -> AuthResult<Self::Passkey> {
        Err(AuthError::NotImplemented("PasskeyOps".to_string()))
    }
    async fn update_passkey_name(&self, _id: &str, _name: &str) -> AuthResult<Self::Passkey> {
        Err(AuthError::NotImplemented("PasskeyOps".to_string()))
    }
    async fn delete_passkey(&self, _id: &str) -> AuthResult<()> { Ok(()) }
}
