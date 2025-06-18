use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::organization::Organization;

/// Enhanced user entity with multi-tenant support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub phone: Option<String>,
    pub timezone: String,
    pub locale: String,
    pub status: UserStatus,
    pub security_settings: SecuritySettings,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// User status in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

/// Security settings for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub mfa_enabled: bool,
    pub mfa_methods: Vec<MfaMethod>,
    pub password_last_changed: DateTime<Utc>,
    pub require_password_change: bool,
    pub login_attempts: u32,
    pub locked_until: Option<DateTime<Utc>>,
    pub trusted_devices: Vec<TrustedDevice>,
}

/// Multi-factor authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    TOTP { secret: String, backup_codes: Vec<String> },
    SMS { phone_number: String },
    Email { email: String },
    WebAuthn { credential_id: String, public_key: String },
}

/// Trusted device for reduced MFA requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub id: Uuid,
    pub name: String,
    pub fingerprint: String,
    pub last_used: DateTime<Utc>,
    pub trusted_until: DateTime<Utc>,
}

/// User preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: Theme,
    pub notifications: NotificationPreferences,
    pub language: String,
    pub date_format: String,
    pub time_format: TimeFormat,
    pub dashboard_layout: DashboardLayout,
}

/// UI theme preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
    pub notification_types: HashMap<NotificationType, bool>,
}

/// Types of notifications
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum NotificationType {
    SecurityAlert,
    BillingUpdate,
    ServiceUpdate,
    MaintenanceNotice,
    QuotaWarning,
    TeamInvitation,
    SystemAlert,
}

/// Time format preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeFormat {
    TwelveHour,
    TwentyFourHour,
}

/// Dashboard layout preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub widgets: Vec<DashboardWidget>,
    pub layout_style: LayoutStyle,
}

/// Dashboard widget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Types of dashboard widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    UsageMetrics,
    RecentActivity,
    QuickActions,
    BillingOverview,
    SystemHealth,
    Notifications,
    TeamActivity,
    CustomChart,
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

/// Widget size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

/// Dashboard layout styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutStyle {
    Grid,
    Masonry,
    Sidebar,
}

/// Organization membership for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMembership {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub role: OrganizationRole,
    pub permissions: Vec<Permission>,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
    pub invitation_status: InvitationStatus,
    pub custom_permissions: HashMap<String, bool>,
}

/// Organization roles with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Developer,
    Analyst,
    Viewer,
    Custom { name: String, permissions: Vec<Permission> },
}

/// Granular permissions within organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    // Organization management
    OrgUpdate,
    OrgDelete,
    OrgInviteUsers,
    OrgManageRoles,
    OrgViewBilling,
    OrgManageBilling,
    OrgViewAudit,
    
    // Database operations
    DatabaseCreate,
    DatabaseDelete,
    DatabaseConfigure,
    DatabaseBackup,
    DatabaseRestore,
    DatabaseViewMetrics,
    
    // Collection operations
    CollectionCreate,
    CollectionDelete,
    CollectionRead,
    CollectionWrite,
    CollectionAdmin,
    
    // Document operations
    DocumentRead,
    DocumentWrite,
    DocumentDelete,
    DocumentAudit,
    
    // Team management
    TeamView,
    TeamInvite,
    TeamManageRoles,
    TeamRemoveUsers,
    
    // Advanced features
    ApiKeyManage,
    WebhookManage,
    IntegrationManage,
    
    // Custom permission
    Custom(String),
}

/// Invitation status for organization membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Declined,
    Expired,
    Canceled,
}

/// User invitation for joining an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInvitation {
    pub id: Uuid,
    pub org_id: Uuid,
    pub inviter_id: Uuid,
    pub invitee_email: String,
    pub role: OrganizationRole,
    pub permissions: Vec<Permission>,
    pub status: InvitationStatus,
    pub invitation_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub message: Option<String>,
}

impl User {
    /// Create a new user with default settings
    pub fn new(email: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email,
            username,
            display_name: None,
            avatar_url: None,
            email_verified: false,
            phone: None,
            timezone: "UTC".to_string(),
            locale: "en-US".to_string(),
            status: UserStatus::PendingVerification,
            security_settings: SecuritySettings::default(),
            preferences: UserPreferences::default(),
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }

    /// Check if user has a specific permission in an organization
    pub fn has_permission(&self, org_id: &Uuid, permission: &Permission, memberships: &[OrganizationMembership]) -> bool {
        if let Some(membership) = memberships.iter().find(|m| &m.org_id == org_id) {
            // Check direct permissions
            if membership.permissions.contains(permission) {
                return true;
            }
            
            // Check role-based permissions
            match &membership.role {
                OrganizationRole::Owner => true, // Owner has all permissions
                OrganizationRole::Admin => {
                    // Admin has most permissions except org deletion
                    !matches!(permission, Permission::OrgDelete)
                }
                OrganizationRole::Developer => {
                    matches!(permission, 
                        Permission::DatabaseCreate | Permission::DatabaseConfigure |
                        Permission::CollectionCreate | Permission::CollectionRead | 
                        Permission::CollectionWrite | Permission::DocumentRead | 
                        Permission::DocumentWrite | Permission::DocumentDelete
                    )
                }
                OrganizationRole::Analyst => {
                    matches!(permission,
                        Permission::CollectionRead | Permission::DocumentRead |
                        Permission::DatabaseViewMetrics
                    )
                }
                OrganizationRole::Viewer => {
                    matches!(permission, Permission::CollectionRead | Permission::DocumentRead)
                }
                OrganizationRole::Custom { permissions, .. } => {
                    permissions.contains(permission)
                }
            }
        } else {
            false
        }
    }

    /// Get all organizations this user belongs to
    pub fn get_organizations(&self, memberships: &[OrganizationMembership]) -> Vec<Uuid> {
        memberships.iter()
            .filter(|m| m.user_id == self.id && m.invitation_status == InvitationStatus::Accepted)
            .map(|m| m.org_id)
            .collect()
    }

    /// Update last login timestamp
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Check if account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.security_settings.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }

    /// Lock account for a duration
    pub fn lock_account(&mut self, duration: chrono::Duration) {
        self.security_settings.locked_until = Some(Utc::now() + duration);
        self.updated_at = Utc::now();
    }

    /// Unlock account
    pub fn unlock_account(&mut self) {
        self.security_settings.locked_until = None;
        self.security_settings.login_attempts = 0;
        self.updated_at = Utc::now();
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            mfa_enabled: false,
            mfa_methods: vec![],
            password_last_changed: Utc::now(),
            require_password_change: false,
            login_attempts: 0,
            locked_until: None,
            trusted_devices: vec![],
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        let mut notification_types = HashMap::new();
        notification_types.insert(NotificationType::SecurityAlert, true);
        notification_types.insert(NotificationType::BillingUpdate, true);
        notification_types.insert(NotificationType::ServiceUpdate, false);
        notification_types.insert(NotificationType::MaintenanceNotice, true);
        notification_types.insert(NotificationType::QuotaWarning, true);
        notification_types.insert(NotificationType::TeamInvitation, true);
        notification_types.insert(NotificationType::SystemAlert, true);
        
        Self {
            theme: Theme::Auto,
            notifications: NotificationPreferences {
                email_notifications: true,
                push_notifications: false,
                sms_notifications: false,
                notification_types,
            },
            language: "en".to_string(),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: TimeFormat::TwentyFourHour,
            dashboard_layout: DashboardLayout {
                widgets: vec![
                    DashboardWidget {
                        id: "usage-metrics".to_string(),
                        widget_type: WidgetType::UsageMetrics,
                        position: WidgetPosition { x: 0, y: 0 },
                        size: WidgetSize { width: 6, height: 4 },
                        settings: HashMap::new(),
                    },
                    DashboardWidget {
                        id: "recent-activity".to_string(),
                        widget_type: WidgetType::RecentActivity,
                        position: WidgetPosition { x: 6, y: 0 },
                        size: WidgetSize { width: 6, height: 4 },
                        settings: HashMap::new(),
                    },
                ],
                layout_style: LayoutStyle::Grid,
            },
        }
    }
}

/// Multi-tenant user management system
pub struct TenantUserManager {
    // In a real implementation, these would be backed by persistent storage
    users: HashMap<Uuid, User>,
    memberships: HashMap<Uuid, Vec<OrganizationMembership>>,
    invitations: HashMap<Uuid, UserInvitation>,
}

impl TenantUserManager {
    /// Create a new tenant user manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            memberships: HashMap::new(),
            invitations: HashMap::new(),
        }
    }

    /// Create a new user
    pub async fn create_user(&mut self, email: String, username: String) -> Result<User> {
        // Check if user already exists
        if self.users.values().any(|u| u.email == email || u.username == username) {
            return Err(anyhow!("User with email or username already exists"));
        }
        
        let user = User::new(email, username);
        let user_id = user.id;
        self.users.insert(user_id, user.clone());
        self.memberships.insert(user_id, vec![]);
        
        Ok(user)
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>> {
        Ok(self.users.get(user_id).cloned())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        Ok(self.users.values().find(|u| u.email == email).cloned())
    }

    /// Update user
    pub async fn update_user(&mut self, user: &User) -> Result<()> {
        if let Some(existing_user) = self.users.get_mut(&user.id) {
            *existing_user = user.clone();
            Ok(())
        } else {
            Err(anyhow!("User not found"))
        }
    }

    /// Delete user
    pub async fn delete_user(&mut self, user_id: &Uuid) -> Result<()> {
        self.users.remove(user_id);
        self.memberships.remove(user_id);
        Ok(())
    }

    /// Invite user to organization
    pub async fn invite_user(&mut self, 
        org_id: Uuid, 
        inviter_id: Uuid, 
        invitee_email: String, 
        role: OrganizationRole
    ) -> Result<UserInvitation> {
        let invitation = UserInvitation {
            id: Uuid::new_v4(),
            org_id,
            inviter_id,
            invitee_email,
            role,
            permissions: vec![], // Will be set based on role
            status: InvitationStatus::Pending,
            invitation_token: Uuid::new_v4().to_string(),
            expires_at: Utc::now() + chrono::Duration::days(7),
            created_at: Utc::now(),
            message: None,
        };
        
        self.invitations.insert(invitation.id, invitation.clone());
        Ok(invitation)
    }

    /// Accept invitation
    pub async fn accept_invitation(&mut self, invitation_id: Uuid, user_id: Uuid) -> Result<()> {
        if let Some(invitation) = self.invitations.get_mut(&invitation_id) {
            if invitation.status == InvitationStatus::Pending && Utc::now() < invitation.expires_at {
                invitation.status = InvitationStatus::Accepted;
                
                // Add user to organization
                let membership = OrganizationMembership {
                    org_id: invitation.org_id,
                    user_id,
                    role: invitation.role.clone(),
                    permissions: invitation.permissions.clone(),
                    joined_at: Utc::now(),
                    invited_by: Some(invitation.inviter_id),
                    invitation_status: InvitationStatus::Accepted,
                    custom_permissions: HashMap::new(),
                };
                
                if let Some(user_memberships) = self.memberships.get_mut(&user_id) {
                    user_memberships.push(membership);
                } else {
                    self.memberships.insert(user_id, vec![membership]);
                }
                
                Ok(())
            } else {
                Err(anyhow!("Invitation is not valid or has expired"))
            }
        } else {
            Err(anyhow!("Invitation not found"))
        }
    }

    /// Get user memberships
    pub async fn get_user_memberships(&self, user_id: &Uuid) -> Result<Vec<OrganizationMembership>> {
        Ok(self.memberships.get(user_id).cloned().unwrap_or_default())
    }

    /// Get organization members
    pub async fn get_organization_members(&self, org_id: &Uuid) -> Result<Vec<(User, OrganizationMembership)>> {
        let mut members = vec![];
        
        for (user_id, memberships) in &self.memberships {
            if let Some(membership) = memberships.iter().find(|m| &m.org_id == org_id) {
                if let Some(user) = self.users.get(user_id) {
                    members.push((user.clone(), membership.clone()));
                }
            }
        }
        
        Ok(members)
    }

    /// Remove user from organization
    pub async fn remove_user_from_organization(&mut self, user_id: &Uuid, org_id: &Uuid) -> Result<()> {
        if let Some(memberships) = self.memberships.get_mut(user_id) {
            memberships.retain(|m| &m.org_id != org_id);
            Ok(())
        } else {
            Err(anyhow!("User not found"))
        }
    }

    /// Remove all users from organization (when org is deleted)
    pub async fn remove_organization_users(&mut self, org_id: &Uuid) -> Result<()> {
        for memberships in self.memberships.values_mut() {
            memberships.retain(|m| &m.org_id != org_id);
        }
        
        // Remove pending invitations for this organization
        self.invitations.retain(|_, inv| &inv.org_id != org_id);
        
        Ok(())
    }

    /// Update user role in organization
    pub async fn update_user_role(&mut self, 
        user_id: &Uuid, 
        org_id: &Uuid, 
        new_role: OrganizationRole
    ) -> Result<()> {
        if let Some(memberships) = self.memberships.get_mut(user_id) {
            if let Some(membership) = memberships.iter_mut().find(|m| &m.org_id == org_id) {
                membership.role = new_role;
                Ok(())
            } else {
                Err(anyhow!("User is not a member of this organization"))
            }
        } else {
            Err(anyhow!("User not found"))
        }
    }
}

impl Default for TenantUserManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_creation() {
        let mut manager = TenantUserManager::new();
        
        let user = manager.create_user(
            "test@example.com".to_string(),
            "testuser".to_string()
        ).await.unwrap();
        
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.username, "testuser");
        assert_eq!(user.status, UserStatus::PendingVerification);
    }

    #[tokio::test]
    async fn test_invitation_workflow() {
        let mut manager = TenantUserManager::new();
        
        // Create users
        let inviter = manager.create_user(
            "admin@company.com".to_string(),
            "admin".to_string()
        ).await.unwrap();
        
        let invitee = manager.create_user(
            "user@company.com".to_string(),
            "user".to_string()
        ).await.unwrap();
        
        let org_id = Uuid::new_v4();
        
        // Send invitation
        let invitation = manager.invite_user(
            org_id,
            inviter.id,
            invitee.email.clone(),
            OrganizationRole::Developer
        ).await.unwrap();
        
        assert_eq!(invitation.status, InvitationStatus::Pending);
        
        // Accept invitation
        manager.accept_invitation(invitation.id, invitee.id).await.unwrap();
        
        // Check membership
        let memberships = manager.get_user_memberships(&invitee.id).await.unwrap();
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].org_id, org_id);
        assert!(matches!(memberships[0].role, OrganizationRole::Developer));
    }

    #[test]
    fn test_permission_checking() {
        let user = User::new("test@example.com".to_string(), "testuser".to_string());
        let org_id = Uuid::new_v4();
        
        let memberships = vec![
            OrganizationMembership {
                org_id,
                user_id: user.id,
                role: OrganizationRole::Developer,
                permissions: vec![],
                joined_at: Utc::now(),
                invited_by: None,
                invitation_status: InvitationStatus::Accepted,
                custom_permissions: HashMap::new(),
            }
        ];
        
        // Developer should have read/write permissions
        assert!(user.has_permission(&org_id, &Permission::DocumentRead, &memberships));
        assert!(user.has_permission(&org_id, &Permission::DocumentWrite, &memberships));
        
        // Developer should not have organization management permissions
        assert!(!user.has_permission(&org_id, &Permission::OrgDelete, &memberships));
    }
}
