use anyhow::Result;
use boa_engine::{
    js_string, native_function::NativeFunction, object::builtins::JsArray, Context, JsError,
    JsObject, JsResult, JsString, JsValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Credential Management API implementation for navigator.credentials
/// https://developer.mozilla.org/en-US/docs/Web/API/Credential_Management_API
pub struct CredentialManager {
    /// Storage for credentials (in-memory for now, can be enhanced with AI memory integration)
    credentials: Arc<Mutex<HashMap<String, StoredCredential>>>,
}

/// Types of credentials supported by the Credential Management API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    Password,
    PublicKey,
    Federated,
    Identity,
}

/// Stored credential representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    pub id: String,
    pub credential_type: CredentialType,
    pub password: Option<String>,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    pub origin: String,
    pub created_at: u64,
}

/// Password credential data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordCredentialData {
    pub id: String,
    pub password: String,
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

/// Federated credential data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedCredentialData {
    pub id: String,
    pub provider: String,
    pub name: Option<String>,
    pub icon_url: Option<String>,
}

/// PublicKey credential creation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialCreationOptions {
    pub challenge: Vec<u8>,
    pub rp: RelyingParty,
    pub user: UserEntity,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: Option<u32>,
    pub exclude_credentials: Option<Vec<PublicKeyCredentialDescriptor>>,
    pub authenticator_selection: Option<AuthenticatorSelectionCriteria>,
    pub attestation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelyingParty {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: Vec<u8>,
    pub name: String,
    pub display_name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubKeyCredParam {
    pub type_: String, // "public-key"
    pub alg: i32,     // COSE algorithm identifier
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredentialDescriptor {
    pub type_: String, // "public-key"
    pub id: Vec<u8>,
    pub transports: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatorSelectionCriteria {
    pub authenticator_attachment: Option<String>, // "platform" | "cross-platform"
    pub require_resident_key: Option<bool>,
    pub resident_key: Option<String>, // "discouraged" | "preferred" | "required"
    pub user_verification: Option<String>, // "required" | "preferred" | "discouraged"
}

impl CredentialManager {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup the navigator.credentials API in the JavaScript context
    pub fn setup_credentials_api(&self, context: &mut Context) -> Result<()> {
        // Get the navigator object
        let navigator = context
            .global_object()
            .get(js_string!("navigator"), context)?;

        if let Ok(navigator_obj) = navigator.try_js_into::<JsObject>() {
            // Create credentials manager object
            let credentials_obj = JsObject::with_object_proto(context.intrinsics());

            // Implement credentials.get()
            let credentials_arc = Arc::clone(&self.credentials);
            let get_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
                Self::credentials_get(Arc::clone(&credentials_arc), args, context)
            });
            credentials_obj.set(js_string!("get"), get_fn, false, context)?;

            // Implement credentials.store()
            let credentials_arc = Arc::clone(&self.credentials);
            let store_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
                Self::credentials_store(Arc::clone(&credentials_arc), args, context)
            });
            credentials_obj.set(js_string!("store"), store_fn, false, context)?;

            // Implement credentials.create()
            let credentials_arc = Arc::clone(&self.credentials);
            let create_fn = NativeFunction::from_fn_ptr(move |_, args, context| {
                Self::credentials_create(Arc::clone(&credentials_arc), args, context)
            });
            credentials_obj.set(js_string!("create"), create_fn, false, context)?;

            // Implement credentials.preventSilentAccess()
            let prevent_silent_access_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
                // For now, just return a resolved promise that invokes the provided resolver
                let promise = context.intrinsics().promise().constructor().call(
                    &JsValue::undefined(),
                    &[NativeFunction::from_fn_ptr(|_, args, context| {
                        if let Some(resolve) = args.get(0) {
                            resolve.as_callable().unwrap().call(
                                &JsValue::undefined(),
                                &[],
                                context,
                            )?;
                        }
                        Ok(JsValue::undefined())
                    }).into()],
                    context,
                )?;
                Ok(promise)
            });
            credentials_obj.set(js_string!("preventSilentAccess"), prevent_silent_access_fn, false, context)?;

            // Set the credentials object on navigator
            navigator_obj.set(js_string!("credentials"), credentials_obj, false, context)?;
        }

        Ok(())
    }

    /// Implementation of navigator.credentials.get()
    fn credentials_get(
        credentials: Arc<Mutex<HashMap<String, StoredCredential>>>,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let options = args.get(0).cloned().unwrap_or(JsValue::undefined());

        // Parse options object
        let mediation = if let Ok(options_obj) = options.try_js_into::<JsObject>() {
            options_obj
                .get(js_string!("mediation"), context)?
                .to_string(context)?
                .to_std_string_escaped()
        } else {
            "optional".to_string()
        };

        // For now, simulate finding a stored credential
        let credential_obj = JsObject::with_object_proto(context.intrinsics());

        // Simulate a password credential
        credential_obj.set(js_string!("id"), js_string!("user@example.com"), false, context)?;
        credential_obj.set(js_string!("type"), js_string!("password"), false, context)?;
        credential_obj.set(js_string!("password"), js_string!("simulated_password"), false, context)?;

        // Return a resolved promise with the credential
        let promise = context.intrinsics().promise().constructor().call(
            &JsValue::undefined(),
            &[NativeFunction::from_fn_ptr(move |_, args, context| {
                if let Some(resolve) = args.get(0) {
                    resolve.as_callable().unwrap().call(
                        &JsValue::undefined(),
                        &[credential_obj.clone().into()],
                        context,
                    )?;
                }
                Ok(JsValue::undefined())
            }).into()],
            context,
        )?;

        Ok(promise)
    }

    /// Implementation of navigator.credentials.store()
    fn credentials_store(
        credentials: Arc<Mutex<HashMap<String, StoredCredential>>>,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let credential = args.get(0).cloned().unwrap_or(JsValue::undefined());

        if let Ok(credential_obj) = credential.try_js_into::<JsObject>() {
            let id = credential_obj
                .get(js_string!("id"), context)?
                .to_string(context)?
                .to_std_string_escaped();

            let type_ = credential_obj
                .get(js_string!("type"), context)?
                .to_string(context)?
                .to_std_string_escaped();

            let password = if type_ == "password" {
                Some(credential_obj
                    .get(js_string!("password"), context)?
                    .to_string(context)?
                    .to_std_string_escaped())
            } else {
                None
            };

            let name = credential_obj
                .get(js_string!("name"), context)
                .ok()
                .and_then(|v| v.to_string(context).ok())
                .map(|s| s.to_std_string_escaped());

            // Store the credential
            let stored_credential = StoredCredential {
                id: id.clone(),
                credential_type: match type_.as_str() {
                    "password" => CredentialType::Password,
                    "public-key" => CredentialType::PublicKey,
                    "federated" => CredentialType::Federated,
                    "identity" => CredentialType::Identity,
                    _ => CredentialType::Password,
                },
                password,
                name,
                icon_url: None,
                origin: "localhost".to_string(), // In real implementation, get from context
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };

            if let Ok(mut creds) = credentials.lock() {
                creds.insert(id, stored_credential);
            }
        }

        // Return a resolved promise with the stored credential
        let promise = context.intrinsics().promise().constructor().call(
            &JsValue::undefined(),
            &[NativeFunction::from_fn_ptr(move |_, args, context| {
                if let Some(resolve) = args.get(0) {
                    resolve.as_callable().unwrap().call(
                        &JsValue::undefined(),
                        &[credential.clone()],
                        context,
                    )?;
                }
                Ok(JsValue::undefined())
            }).into()],
            context,
        )?;

        Ok(promise)
    }

    /// Implementation of navigator.credentials.create()
    fn credentials_create(
        _credentials: Arc<Mutex<HashMap<String, StoredCredential>>>,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let options = args.get(0).cloned().unwrap_or(JsValue::undefined());

        // For now, create a mock credential based on the options
        let credential_obj = JsObject::with_object_proto(context.intrinsics());

        if let Ok(options_obj) = options.try_js_into::<JsObject>() {
            // Check if this is a password credential creation
            if let Ok(password_opts) = options_obj.get(js_string!("password"), context) {
                if !password_opts.is_undefined() {
                    credential_obj.set(js_string!("type"), js_string!("password"), false, context)?;
                    credential_obj.set(js_string!("id"), js_string!("generated_user_id"), false, context)?;
                    credential_obj.set(js_string!("password"), js_string!("generated_password"), false, context)?;
                }
            }

            // Check if this is a public key credential creation
            if let Ok(publickey_opts) = options_obj.get(js_string!("publicKey"), context) {
                if !publickey_opts.is_undefined() {
                    credential_obj.set(js_string!("type"), js_string!("public-key"), false, context)?;
                    credential_obj.set(js_string!("id"), js_string!("generated_credential_id"), false, context)?;

                    // Mock authenticator response
                    let response_obj = JsObject::with_object_proto(context.intrinsics());
                    response_obj.set(js_string!("clientDataJSON"), js_string!("mock_client_data"), false, context)?;
                    response_obj.set(js_string!("attestationObject"), js_string!("mock_attestation"), false, context)?;
                    credential_obj.set(js_string!("response"), response_obj, false, context)?;
                }
            }
        }

        // Return a resolved promise with the created credential
        let promise = context.intrinsics().promise().constructor().call(
            &JsValue::undefined(),
            &[NativeFunction::from_fn_ptr(move |_, args, context| {
                if let Some(resolve) = args.get(0) {
                    resolve.as_callable().unwrap().call(
                        &JsValue::undefined(),
                        &[credential_obj.clone().into()],
                        context,
                    )?;
                }
                Ok(JsValue::undefined())
            }).into()],
            context,
        )?;

        Ok(promise)
    }

    /// Get all stored credentials (for debugging/management)
    pub fn get_all_credentials(&self) -> HashMap<String, StoredCredential> {
        self.credentials.lock().unwrap_or_default().clone()
    }

    /// Clear all stored credentials
    pub fn clear_credentials(&self) {
        if let Ok(mut creds) = self.credentials.lock() {
            creds.clear();
        }
    }

    /// Get credentials for a specific origin
    pub fn get_credentials_for_origin(&self, origin: &str) -> Vec<StoredCredential> {
        if let Ok(creds) = self.credentials.lock() {
            creds
                .values()
                .filter(|cred| cred.origin == origin)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Store a credential programmatically (useful for integration with AI memory)
    pub fn store_credential(&self, credential: StoredCredential) {
        if let Ok(mut creds) = self.credentials.lock() {
            creds.insert(credential.id.clone(), credential);
        }
    }

    /// Remove a credential by ID
    pub fn remove_credential(&self, id: &str) -> bool {
        if let Ok(mut creds) = self.credentials.lock() {
            creds.remove(id).is_some()
        } else {
            false
        }
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_manager_creation() {
        let manager = CredentialManager::new();
        assert_eq!(manager.get_all_credentials().len(), 0);
    }

    #[test]
    fn test_store_and_retrieve_credential() {
        let manager = CredentialManager::new();

        let credential = StoredCredential {
            id: "test@example.com".to_string(),
            credential_type: CredentialType::Password,
            password: Some("test_password".to_string()),
            name: Some("Test User".to_string()),
            icon_url: None,
            origin: "https://example.com".to_string(),
            created_at: 1234567890,
        };

        manager.store_credential(credential.clone());

        let stored = manager.get_all_credentials();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored.get("test@example.com").unwrap().password, credential.password);
    }

    #[test]
    fn test_remove_credential() {
        let manager = CredentialManager::new();

        let credential = StoredCredential {
            id: "test@example.com".to_string(),
            credential_type: CredentialType::Password,
            password: Some("test_password".to_string()),
            name: Some("Test User".to_string()),
            icon_url: None,
            origin: "https://example.com".to_string(),
            created_at: 1234567890,
        };

        manager.store_credential(credential);
        assert_eq!(manager.get_all_credentials().len(), 1);

        let removed = manager.remove_credential("test@example.com");
        assert!(removed);
        assert_eq!(manager.get_all_credentials().len(), 0);
    }

    #[test]
    fn test_get_credentials_for_origin() {
        let manager = CredentialManager::new();

        let credential1 = StoredCredential {
            id: "user1@example.com".to_string(),
            credential_type: CredentialType::Password,
            password: Some("password1".to_string()),
            name: Some("User 1".to_string()),
            icon_url: None,
            origin: "https://example.com".to_string(),
            created_at: 1234567890,
        };

        let credential2 = StoredCredential {
            id: "user2@other.com".to_string(),
            credential_type: CredentialType::Password,
            password: Some("password2".to_string()),
            name: Some("User 2".to_string()),
            icon_url: None,
            origin: "https://other.com".to_string(),
            created_at: 1234567891,
        };

        manager.store_credential(credential1);
        manager.store_credential(credential2);

        let example_creds = manager.get_credentials_for_origin("https://example.com");
        assert_eq!(example_creds.len(), 1);
        assert_eq!(example_creds[0].id, "user1@example.com");

        let other_creds = manager.get_credentials_for_origin("https://other.com");
        assert_eq!(other_creds.len(), 1);
        assert_eq!(other_creds[0].id, "user2@other.com");
    }
}