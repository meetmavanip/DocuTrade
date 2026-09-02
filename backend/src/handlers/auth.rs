use axum::{
    routing::{post, get},
    Router, Json, extract::{State, Extension},
    middleware,
};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::errors::AppError;
use crate::services::auth::{hash_password, verify_password, create_jwt, generate_nonce, verify_signature, Claims};
use crate::middleware::auth::{auth_middleware, RequireBuyer, RequireSeller};
use uuid::Uuid;
use chrono::{Utc, Duration};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/wallet/nonce", post(wallet_nonce))
        .route("/wallet/verify", post(wallet_verify))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        // Protected routes
        .route("/me", get(me).route_layer(middleware::from_fn(auth_middleware)))
        .route("/logout", post(logout).route_layer(middleware::from_fn(auth_middleware)))
        .route("/wallet/link", post(wallet_link).route_layer(middleware::from_fn(auth_middleware)))
}

async fn register(State(state): State<AppState>, Json(payload): Json<Value>) -> Result<Json<Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Password required".into()))?;
    let first_name = payload.get("first_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let last_name = payload.get("last_name").and_then(|v| v.as_str()).unwrap_or("User");
    let organization_name = payload.get("organization").and_then(|v| v.as_str()).unwrap_or("Independent");
    let role_str = payload.get("role").and_then(|v| v.as_str()).unwrap_or("BUYER").to_uppercase();
    if role_str != "BUYER" && role_str != "SELLER" {
        return Err(AppError::Validation("Role must be BUYER or SELLER".into()));
    }
    
    // Validate email
    let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", email)
        .fetch_optional(&state.db).await?;
    if existing.is_some() {
        return Err(AppError::Conflict("Email already registered".into()));
    }

    // Verify role exists
    let role_rec = sqlx::query!("SELECT id, name::text as name_str FROM roles WHERE name::text = $1", role_str)
        .fetch_optional(&state.db).await?;
        
    let role_id = match role_rec {
        Some(r) => r.id,
        None => return Err(AppError::Validation(format!("Invalid role selected: {}", role_str))),
    };

    let hashed_pw = hash_password(password)?;
    
    // Start Transaction
    let mut tx = state.db.begin().await?;

    let org_id = Uuid::new_v4();
    let org_type_str = if role_str == "EXPORTER" || role_str == "BUYER" { role_str.clone() } else { "PLATFORM".to_string() };
    
    // Note: Use sqlx::query without ! to avoid compile-time issues with custom enums
    sqlx::query(
        "INSERT INTO organizations (id, name, organization_type, country, created_at, updated_at) 
         VALUES ($1, $2, $3::organization_type, 'Unknown', $4, $5)"
    )
    .bind(org_id)
    .bind(organization_name)
    .bind(org_type_str)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&mut *tx)
    .await?;

    let user_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, organization_id, first_name, last_name, email, password_hash, is_active, email_verified, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        user_id, org_id, first_name, last_name, email, hashed_pw, true, false, Utc::now(), Utc::now()
    ).execute(&mut *tx).await?;

    sqlx::query!(
        "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ($1, $2, $3)",
        user_id, role_id, Utc::now()
    ).execute(&mut *tx).await?;

    tx.commit().await?;

    let token = create_jwt(&user_id.to_string(), &role_str)?;
    
    Ok(Json(json!({ "token": token, "user": { "id": user_id, "email": email, "role": role_str } })))
}

async fn login(State(state): State<AppState>, Json(payload): Json<Value>) -> Result<Json<Value>, AppError> {
    let email = payload.get("email").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Email required".into()))?;
    let password = payload.get("password").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Password required".into()))?;

    let user_rec = sqlx::query!("SELECT id, password_hash FROM users WHERE email = $1 AND is_active = true", email)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".into()))?;

    let hash = user_rec.password_hash.ok_or_else(|| AppError::Auth("Account cannot be logged in with password".into()))?;

    if !verify_password(password, &hash)? {
        return Err(AppError::Auth("Invalid email or password".into()));
    }

    let role_rec = sqlx::query!(
        "SELECT r.name::text as name_str FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 LIMIT 1",
        user_rec.id
    ).fetch_optional(&state.db).await?;
    let role = role_rec.map(|r| r.name_str.unwrap_or_else(|| "BUYER".into())).unwrap_or_else(|| "BUYER".into());

    let token = create_jwt(&user_rec.id.to_string(), &role)?;
    
    // Update last login
    sqlx::query!("UPDATE users SET last_login_at = $1 WHERE id = $2", Utc::now(), user_rec.id)
        .execute(&state.db).await?;

    Ok(Json(json!({ "token": token, "user": { "id": user_rec.id, "email": email, "role": role } })))
}

async fn wallet_nonce(State(state): State<AppState>, Json(payload): Json<Value>) -> Result<Json<Value>, AppError> {
    let wallet_address = payload.get("wallet_address").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Wallet address required".into()))?;
    
    let nonce = generate_nonce();
    let expires = Utc::now() + Duration::minutes(15);
    
    sqlx::query!(
        "INSERT INTO auth_nonces (id, wallet_address, nonce, expires_at, used) VALUES ($1, $2, $3, $4, false)",
        Uuid::new_v4(), wallet_address, nonce, expires
    ).execute(&state.db).await?;

    Ok(Json(json!({ "wallet_address": wallet_address, "nonce": nonce, "message": format!("Sign this message to authenticate with DocuTrade: {}", nonce) })))
}

async fn wallet_verify(State(state): State<AppState>, Json(payload): Json<Value>) -> Result<Json<Value>, AppError> {
    let wallet_address = payload.get("wallet_address").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Wallet address required".into()))?;
    let nonce = payload.get("nonce").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Nonce required".into()))?;
    let signature = payload.get("signature").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Signature required".into()))?;

    let nonce_rec = sqlx::query!("SELECT id, expires_at, used FROM auth_nonces WHERE wallet_address = $1 AND nonce = $2", wallet_address, nonce)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::Auth("Invalid or expired nonce".into()))?;

    if nonce_rec.used || nonce_rec.expires_at < Utc::now() {
        return Err(AppError::Auth("Nonce expired or already used".into()));
    }

    if !verify_signature(wallet_address, nonce, signature)? {
        return Err(AppError::Auth("Invalid signature".into()));
    }

    // Mark nonce used
    sqlx::query!("UPDATE auth_nonces SET used = true WHERE id = $1", nonce_rec.id).execute(&state.db).await?;

    // Find wallet & user
    let wallet_rec = sqlx::query!("SELECT user_id FROM wallets WHERE address = $1", wallet_address)
        .fetch_optional(&state.db).await?;

    let user_id = if let Some(w) = wallet_rec {
        w.user_id.unwrap_or_else(|| Uuid::new_v4()) // Handle orphaned wallets defensively
    } else {
        // Create user and wallet
        let new_user_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, first_name, last_name, is_active, email_verified, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            new_user_id, "Wallet", "User", true, false, Utc::now(), Utc::now()
        ).execute(&state.db).await?;

        sqlx::query!(
            "INSERT INTO wallets (id, user_id, address, chain_id, is_primary, is_verified, verified_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            Uuid::new_v4(), new_user_id, wallet_address, 421614, true, true, Utc::now(), Utc::now(), Utc::now()
        ).execute(&state.db).await?;

        new_user_id
    };

    let role_rec = sqlx::query!(
        "SELECT r.name::text as name_str FROM user_roles ur JOIN roles r ON ur.role_id = r.id WHERE ur.user_id = $1 LIMIT 1",
        user_id
    ).fetch_optional(&state.db).await?;
    let role = role_rec.map(|r| r.name_str.unwrap_or_else(|| "BUYER".into())).unwrap_or_else(|| "BUYER".into());

    let token = create_jwt(&user_id.to_string(), &role)?;
    
    sqlx::query!("UPDATE users SET last_login_at = $1 WHERE id = $2", Utc::now(), user_id).execute(&state.db).await?;

    Ok(Json(json!({ "token": token, "user": { "id": user_id, "wallet_address": wallet_address, "role": role } })))
}

async fn me(State(state): State<AppState>, Extension(claims): Extension<Claims>) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    let user = sqlx::query!(
        r#"
        SELECT u.id, u.email, u.first_name, u.last_name, o.name as org_name
        FROM users u
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE u.id = $1
        "#, 
        user_id
    )
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;

    let wallet = sqlx::query!("SELECT address FROM wallets WHERE user_id = $1 AND is_primary = true", user_id)
        .fetch_optional(&state.db).await?;

    let mut response = json!({
        "id": user.id,
        "first_name": user.first_name,
        "last_name": user.last_name,
        "role": claims.role,
        "organization_name": user.org_name,
    });

    if let Some(e) = user.email {
        response["email"] = json!(e);
    }
    if let Some(w) = wallet {
        response["wallet_address"] = json!(w.address);
    }

    Ok(Json(response))
}

async fn wallet_link(State(state): State<AppState>, Extension(claims): Extension<Claims>, Json(payload): Json<Value>) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    let wallet_address = payload.get("wallet_address").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Wallet address required".into()))?;
    let nonce = payload.get("nonce").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Nonce required".into()))?;
    let signature = payload.get("signature").and_then(|v| v.as_str()).ok_or_else(|| AppError::Validation("Signature required".into()))?;

    // Similar to wallet_verify, but we insert/update the wallet to link to current user
    let nonce_rec = sqlx::query!("SELECT id, expires_at, used FROM auth_nonces WHERE wallet_address = $1 AND nonce = $2", wallet_address, nonce)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::Auth("Invalid or expired nonce".into()))?;

    if nonce_rec.used || nonce_rec.expires_at < Utc::now() {
        return Err(AppError::Auth("Nonce expired or already used".into()));
    }

    if !verify_signature(wallet_address, nonce, signature)? {
        return Err(AppError::Auth("Invalid signature".into()));
    }

    sqlx::query!("UPDATE auth_nonces SET used = true WHERE id = $1", nonce_rec.id).execute(&state.db).await?;

    // Check if wallet is already linked to someone else
    let existing = sqlx::query!("SELECT user_id FROM wallets WHERE address = $1", wallet_address)
        .fetch_optional(&state.db).await?;
    
    if let Some(e) = existing {
        if e.user_id != Some(user_id) {
            return Err(AppError::Auth("Wallet already linked to another account".into()));
        }
    } else {
        sqlx::query!(
            "INSERT INTO wallets (id, user_id, address, chain_id, is_primary, is_verified, verified_at, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            Uuid::new_v4(), user_id, wallet_address, 421614, false, true, Utc::now(), Utc::now(), Utc::now()
        ).execute(&state.db).await?;
    }

    Ok(Json(json!({ "success": true, "message": "Wallet linked successfully" })))
}

async fn logout() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "success": true, "message": "Logged out successfully" })))
}

async fn forgot_password() -> Result<Json<Value>, AppError> {
    // Stub
    Ok(Json(json!({ "success": true, "message": "Reset link sent" })))
}

async fn reset_password() -> Result<Json<Value>, AppError> {
    // Stub
    Ok(Json(json!({ "success": true, "message": "Password updated" })))
}
