// Authentication Module

use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use bcrypt;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: i64,
    iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
    user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    username: String,
    role: String,
    last_login: Option<DateTime<Utc>>,
}

const JWT_SECRET: &[u8] = b"your-secret-key-change-in-production";

#[tauri::command]
pub async fn login(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    username: String,
    password: String,
) -> Result<LoginResponse, String> {
    let app_state = state.lock().await;
    let mut users = app_state.users.write().await;
    
    // Check demo mode
    let is_demo = *app_state.is_demo_mode.read().await;
    
    if is_demo {
        // In demo mode, accept any credentials
        let user = User {
            username: username.clone(),
            password_hash: String::new(),
            role: if username == "admin" { "admin" } else { "operator" }.to_string(),
            created_at: Utc::now(),
            last_login: Some(Utc::now()),
        };
        
        users.insert(username.clone(), user.clone());
        
        let token = generate_token(&username, &user.role)
            .map_err(|e| e.to_string())?;
        
        return Ok(LoginResponse {
            token,
            user: UserInfo {
                username: user.username,
                role: user.role,
                last_login: user.last_login,
            },
        });
    }
    
    // Normal authentication
    let user = users.get_mut(&username)
        .ok_or_else(|| "Invalid username or password".to_string())?;
    
    // Verify password
    if !bcrypt::verify(&password, &user.password_hash)
        .map_err(|e| format!("Authentication failed: {}", e))? {
        return Err("Invalid username or password".to_string());
    }
    
    // Update last login
    user.last_login = Some(Utc::now());
    
    // Generate JWT token
    let token = generate_token(&username, &user.role)
        .map_err(|e| e.to_string())?;
    
    Ok(LoginResponse {
        token,
        user: UserInfo {
            username: user.username.clone(),
            role: user.role.clone(),
            last_login: user.last_login,
        },
    })
}

#[tauri::command]
pub async fn logout() -> Result<(), String> {
    // Token invalidation would be handled client-side
    // Server-side could maintain a blacklist if needed
    Ok(())
}

#[tauri::command]
pub async fn get_current_user(token: String) -> Result<UserInfo, String> {
    let claims = verify_token(&token)
        .map_err(|e| e.to_string())?;
    
    Ok(UserInfo {
        username: claims.sub,
        role: claims.role,
        last_login: None,
    })
}

#[tauri::command]
pub async fn update_password(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    token: String,
    old_password: String,
    new_password: String,
) -> Result<(), String> {
    let claims = verify_token(&token)
        .map_err(|e| e.to_string())?;
    
    let app_state = state.lock().await;
    let mut users = app_state.users.write().await;
    
    let user = users.get_mut(&claims.sub)
        .ok_or_else(|| "User not found".to_string())?;
    
    // Verify old password
    if !bcrypt::verify(&old_password, &user.password_hash)
        .map_err(|e| format!("Password verification failed: {}", e))? {
        return Err("Invalid old password".to_string());
    }
    
    // Hash new password
    user.password_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    Ok(())
}

fn generate_token(username: &str, role: &str) -> Result<String> {
    let now = Utc::now();
    let exp = (now + Duration::hours(24)).timestamp();
    
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        exp,
        iat: now.timestamp(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    ).map_err(|e| anyhow!("Failed to generate token: {}", e))
}

fn verify_token(token: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| anyhow!("Invalid token: {}", e))
}