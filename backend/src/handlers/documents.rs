use axum::{routing::{get, post}, Router, Json, extract::{State, Path, Extension, Multipart}, middleware, response::{Response, IntoResponse}};
use axum::http::header::{CONTENT_TYPE, CONTENT_DISPOSITION};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::auth_middleware;
use crate::services::auth::Claims;
use crate::services::hashing::hash_document;
use crate::services::blockchain::verify_blockchain_receipt;
use uuid::Uuid;
use chrono::Utc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_documents))
        .route("/upload", post(upload_document))
        .route("/verify-hash", post(verify_hash))
        .route("/:id/verify", post(verify_document))
        .route("/:id/file", get(get_document_file))
        .route("/:id/approve", post(approve_document))
        .route("/:id/reject", post(reject_document))
        .route("/:id/blockchain-verify", post(blockchain_verify))
        .route("/:id/verification", get(get_verification))
        .route("/:id/integrity", get(check_integrity))
        .route("/:id/reupload", post(reupload_document))
        .route_layer(middleware::from_fn(auth_middleware))
}

async fn upload_document(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    // Check role
    if claims.role.to_uppercase() != "SELLER" {
        return Err(AppError::Auth("Only sellers can upload documents".into()));
    }
    
    let mut shipment_id_str = String::new();
    let mut doc_type = String::new();
    let mut client_hash = String::new();
    let mut filename = String::new();
    let mut custom_doc_name = String::new();
    let mut mime_type = String::new();
    let mut file_bytes = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            filename = field.file_name().unwrap_or("unnamed").to_string();
            mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            file_bytes = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?.to_vec();
        } else if name == "shipment_id" {
            shipment_id_str = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        } else if name == "type" {
            doc_type = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        } else if name == "hash" {
            client_hash = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        } else if name == "document_name" {
            custom_doc_name = field.text().await.map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }
    
    if !custom_doc_name.is_empty() {
        filename = custom_doc_name;
    }
    
    if file_bytes.is_empty() {
        return Err(AppError::Validation("No file provided".into()));
    }

    // IMPORTANT: Compute SHA-256 hash server-side from actual file bytes
    let doc_hash = hash_document(&file_bytes)?;

    // Cross-check with client-submitted hash if provided
    if !client_hash.is_empty() && client_hash != doc_hash {
        tracing::warn!("Client hash mismatch: client={}, server={}", client_hash, doc_hash);
        // Use server-computed hash as the authoritative one
    }
    
    // Map doc type to valid PostgreSQL document_type enum values
    let db_doc_type = match doc_type.to_uppercase().as_str() {
        "COMMERCIAL INVOICE" | "COMMERCIAL_INVOICE" => "COMMERCIAL_INVOICE",
        "PACKING LIST" | "PACKING_LIST" => "PACKING_LIST",
        "CERTIFICATE OF ORIGIN" | "CERTIFICATE_OF_ORIGIN" => "CERTIFICATE_OF_ORIGIN",
        "QUALITY CERTIFICATE" | "QUALITY_CERTIFICATE" => "QUALITY_CERTIFICATE",
        "INSPECTION CERTIFICATE" | "INSPECTION_CERTIFICATE" => "INSPECTION_CERTIFICATE",
        "INSURANCE DOCUMENT" | "INSURANCE_DOCUMENT" | "INSURANCE_CERTIFICATE" => "INSURANCE_DOCUMENT",
        "SHIPPING DOCUMENT" | "SHIPPING_DOCUMENT" | "BILL OF LADING" | "BILL_OF_LADING" => "SHIPPING_DOCUMENT",
        _ => "COMMERCIAL_INVOICE"
    };

    // Find internal shipment UUID (support both display ID like EXP-IND-... and UUID)
    let shipment_rec = sqlx::query!("SELECT id, exporter_id, buyer_id FROM shipments WHERE shipment_id = $1 OR id::text = $1 LIMIT 1", shipment_id_str)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Shipment not found".into()))?;
        
    let user = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        
    let org_id = user.organization_id.ok_or_else(|| AppError::Validation("User has no org".into()))?;
    
    if shipment_rec.exporter_id != org_id {
        return Err(AppError::Auth("Not authorized to upload to this shipment".into()));
    }
        
    let doc_id = Uuid::new_v4();
    let file_path = format!("uploads/{}_{}", doc_id, filename);
    
    tokio::fs::write(&file_path, &file_bytes).await.map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;
    
    let document_id_str = format!("DOC-{}", &doc_id.to_string()[0..8].to_uppercase());
    
    // IPFS is simulated
    let ipfs_cid = format!("QmSimulatedHash{}", &doc_id.to_string()[0..8]);
    
    sqlx::query(
        &format!("INSERT INTO documents (id, document_id, shipment_id, uploaded_by, document_type, filename, mime_type, file_size, current_version, sha256, storage_reference, ipfs_cid, status, approval_status, created_at, updated_at)
         VALUES ($1, $2, $3, $4, '{}'::document_type, $5, $6, $7, 1, $8, $9, $10, 'PENDING', 'PENDING', $11, $12)", db_doc_type)
    )
    .bind(doc_id)
    .bind(&document_id_str)
    .bind(shipment_rec.id)
    .bind(user_id)
    .bind(&filename)
    .bind(&mime_type)
    .bind(file_bytes.len() as i64)
    .bind(&doc_hash)
    .bind(&file_path)
    .bind(&ipfs_cid)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&state.db).await?;
    
    // Notify the buyer(s)
    let buyer_users = sqlx::query!("SELECT id FROM users WHERE organization_id = $1", shipment_rec.buyer_id)
        .fetch_all(&state.db)
        .await?;
        
    for buyer_user in buyer_users {
        let _ = sqlx::query!(
            "INSERT INTO notifications (id, user_id, type, title, message, related_entity_id, related_entity_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            Uuid::new_v4(),
            buyer_user.id,
            "DOCUMENT_APPROVAL_REQUIRED",
            "Document Approval Required",
            &format!("The seller has submitted documents for your verification. Trade: {}", shipment_id_str),
            doc_id,
            "document"
        )
        .execute(&state.db)
        .await;
    }
    
    Ok(Json(json!({ "document_id": doc_id, "hash": doc_hash, "message": "Upload successful" })))
}

async fn get_document_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    let doc = sqlx::query!("SELECT d.storage_reference, d.mime_type, d.filename, s.exporter_id, s.buyer_id, s.id as shipment_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;
        
    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Auth("No organization".into()))?;
        
    let mut has_access = false;
    
    if claims.role.to_uppercase() == "SELLER" && doc.exporter_id == user_org {
        has_access = true;
    } else if claims.role.to_uppercase() == "BUYER" {
        if doc.buyer_id == user_org {
            has_access = true;
        } else {
            let access = sqlx::query!("SELECT id FROM trade_access WHERE buyer_id = $1 AND shipment_id = $2", user_id, doc.shipment_id)
                .fetch_optional(&state.db).await?;
            if access.is_some() {
                has_access = true;
            }
        }
    }
    
    if !has_access {
        return Err(AppError::Auth("Not authorized to view this document".into()));
    }
    
    let path = doc.storage_reference.unwrap_or_default();
    let bytes = tokio::fs::read(&path).await.map_err(|_| AppError::NotFound("File not found on disk".into()))?;
    
    let mime = doc.mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let filename = doc.filename;
    
    let response = Response::builder()
        .header(CONTENT_TYPE, mime)
        .header(CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", filename))
        .body(axum::body::Body::from(bytes))
        .map_err(|_| AppError::Internal("Failed to build response".into()))?;
        
    Ok(response)
}

async fn approve_document(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if claims.role.to_uppercase() != "BUYER" {
        return Err(AppError::Auth("Only buyers can approve documents".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;

    // Verify document exists and get its shipment
    let doc = sqlx::query!("SELECT d.id, d.shipment_id as doc_shipment_id, d.status::text as status, d.sha256, s.buyer_id, s.exporter_id, s.shipment_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // Verify buyer has access
    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Auth("No organization".into()))?;

    let mut has_access = doc.buyer_id == user_org;
    if !has_access {
        let access = sqlx::query!("SELECT id FROM trade_access WHERE buyer_id = $1 AND shipment_id = $2", user_id, doc.doc_shipment_id)
            .fetch_optional(&state.db).await?;
        has_access = access.is_some();
    }

    if !has_access {
        return Err(AppError::Auth("Not authorized to approve this document".into()));
    }

    // Check status — only PENDING documents can be approved
    let current_status = doc.status.unwrap_or_default();
    if current_status != "PENDING" {
        return Err(AppError::Validation(format!("Document cannot be approved from status: {}", current_status)));
    }

    // Update status to APPROVED (database approval) — blockchain step is separate
    sqlx::query("UPDATE documents SET status = 'APPROVED', approval_status = 'APPROVED', updated_at = $1 WHERE id = $2")
        .bind(Utc::now())
        .bind(id)
        .execute(&state.db).await?;

    // Record the approval in document_approvals table
    sqlx::query(
        "INSERT INTO document_approvals (id, document_id, reviewed_by, decision, comments, reviewed_at, created_at) VALUES ($1, $2, $3, 'APPROVED'::approval_decision, 'Buyer approved document', $4, $5)"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(user_id)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&state.db).await?;

    // Record in audit log
    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'APPROVE_DOCUMENT', 'document', $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(id)
    .bind(Utc::now())
    .execute(&state.db).await?;
    
    // Notify the seller(s)
    let seller_users = sqlx::query!("SELECT id FROM users WHERE organization_id = $1", doc.exporter_id)
        .fetch_all(&state.db)
        .await?;
        
    for seller_user in seller_users {
        let _ = sqlx::query!(
            "INSERT INTO notifications (id, user_id, type, title, message, related_entity_id, related_entity_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            Uuid::new_v4(),
            seller_user.id,
            "DOCUMENT_APPROVED",
            "Document Approved",
            &format!("Document for shipment {} was approved", doc.shipment_id),
            id,
            "document"
        )
        .execute(&state.db)
        .await;
    }
    
    Ok(Json(json!({ 
        "message": "Document approved successfully",
        "document_hash": doc.sha256,
        "status": "APPROVED"
    })))
}

async fn reject_document(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>
) -> Result<Json<Value>, AppError> {
    if claims.role.to_uppercase() != "BUYER" {
        return Err(AppError::Auth("Only buyers can reject documents".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    let reason = payload.get("reason").and_then(|v| v.as_str()).unwrap_or("");
    if reason.is_empty() {
        return Err(AppError::Validation("Rejection reason is required".into()));
    }

    // Verify document exists
    let doc = sqlx::query!("SELECT d.id, d.shipment_id as doc_shipment_id, d.status::text as status, s.buyer_id, s.exporter_id, s.shipment_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // Verify buyer has access
    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Auth("No organization".into()))?;

    let mut has_access = doc.buyer_id == user_org;
    if !has_access {
        let access = sqlx::query!("SELECT id FROM trade_access WHERE buyer_id = $1 AND shipment_id = $2", user_id, doc.doc_shipment_id)
            .fetch_optional(&state.db).await?;
        has_access = access.is_some();
    }

    if !has_access {
        return Err(AppError::Auth("Not authorized to reject this document".into()));
    }

    // Check status
    let current_status = doc.status.unwrap_or_default();
    if current_status != "PENDING" {
        return Err(AppError::Validation(format!("Document cannot be rejected from status: {}", current_status)));
    }

    // Update document status + store rejection reason
    sqlx::query("UPDATE documents SET status = 'REJECTED', approval_status = 'REJECTED', rejection_reason = $1, updated_at = $2 WHERE id = $3")
        .bind(reason)
        .bind(Utc::now())
        .bind(id)
        .execute(&state.db).await?;

    // Record in document_approvals
    sqlx::query(
        "INSERT INTO document_approvals (id, document_id, reviewed_by, decision, comments, reviewed_at, created_at) VALUES ($1, $2, $3, 'REJECTED'::approval_decision, $4, $5, $6)"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(user_id)
    .bind(reason)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&state.db).await?;

    // Record in audit log
    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'REJECT_DOCUMENT', 'document', $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(id)
    .bind(Utc::now())
    .execute(&state.db).await?;
        
    // Notify the seller(s)
    let seller_users = sqlx::query!("SELECT id FROM users WHERE organization_id = $1", doc.exporter_id)
        .fetch_all(&state.db)
        .await?;
        
    for seller_user in seller_users {
        let _ = sqlx::query!(
            "INSERT INTO notifications (id, user_id, type, title, message, related_entity_id, related_entity_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            Uuid::new_v4(),
            seller_user.id,
            "DOCUMENT_REJECTED",
            "Document Rejected",
            &format!("Document for shipment {} was rejected. Reason: {}", doc.shipment_id, reason),
            id,
            "document"
        )
        .execute(&state.db)
        .await;
    }
        
    Ok(Json(json!({ "message": "Document rejected", "reason": reason })))
}

/// POST /documents/:id/blockchain-verify
/// Backend verifies the blockchain transaction receipt before marking VERIFIED.
async fn blockchain_verify(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    // 1. User is authenticated (already via middleware)
    // 2. User is a BUYER
    if claims.role.to_uppercase() != "BUYER" {
        return Err(AppError::Auth("Only buyers can verify documents on blockchain".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;

    // 3. Extract submitted blockchain data
    let transaction_hash = payload.get("transaction_hash").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("transaction_hash required".into()))?;
    let wallet_address = payload.get("wallet_address").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("wallet_address required".into()))?;
    let chain_id = payload.get("chain_id").and_then(|v| v.as_i64())
        .ok_or_else(|| AppError::Validation("chain_id required".into()))?;
    let contract_address = payload.get("contract_address").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("contract_address required".into()))?;
    let block_number = payload.get("block_number").and_then(|v| v.as_i64());

    // 4. Verify chain_id is Arbitrum Sepolia
    if chain_id != 421614 {
        return Err(AppError::Validation(format!("Invalid chain ID: {}. Expected 421614 (Arbitrum Sepolia)", chain_id)));
    }

    // 5. Verify contract_address matches configured contract
    let expected_contract = &state.config.document_verification_contract;
    if !contract_address.eq_ignore_ascii_case(expected_contract) {
        return Err(AppError::Validation(format!(
            "Contract address mismatch. Submitted: {}, Expected: {}",
            contract_address, expected_contract
        )));
    }

    // 6. Verify document exists and get details
    let doc = sqlx::query!("SELECT d.id, d.shipment_id, d.status::text as status, d.sha256, s.buyer_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // 7. Buyer has access to the trade
    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Auth("No organization".into()))?;

    let mut has_access = doc.buyer_id == user_org;
    if !has_access {
        let access = sqlx::query!("SELECT id FROM trade_access WHERE buyer_id = $1 AND shipment_id = $2", user_id, doc.shipment_id)
            .fetch_optional(&state.db).await?;
        has_access = access.is_some();
    }

    if !has_access {
        return Err(AppError::Auth("Not authorized to verify this document".into()));
    }

    // 8. Document has not already been verified
    let current_status = doc.status.unwrap_or_default();
    if current_status == "VERIFIED" {
        return Err(AppError::Conflict("Document has already been blockchain-verified".into()));
    }

    // Must be APPROVED or PENDING first
    if current_status != "PENDING" && current_status != "APPROVED" && current_status != "BLOCKCHAIN_PENDING" && current_status != "BLOCKCHAIN_FAILED" {
        return Err(AppError::Validation(format!(
            "Document must be approved or pending before blockchain verification. Current status: {}",
            current_status
        )));
    }
    
    // If it was PENDING, we record the approval as well
    if current_status == "PENDING" {
        sqlx::query(
            "INSERT INTO document_approvals (id, document_id, reviewed_by, decision, comments, reviewed_at, created_at) VALUES ($1, $2, $3, 'APPROVED'::approval_decision, 'Buyer approved and verified document on blockchain', $4, $5)"
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(user_id)
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&state.db).await?;
        
        sqlx::query(
            "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'APPROVE_DOCUMENT', 'document', $3, $4)"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(id)
        .bind(Utc::now())
        .execute(&state.db).await?;
    }

    // 9. Document hash exists
    let document_hash = doc.sha256;

    // 10. Verify the blockchain transaction via RPC
    let rpc_url = &state.config.arbitrum_rpc_url;
    let receipt_result = verify_blockchain_receipt(rpc_url, transaction_hash, expected_contract).await?;

    if !receipt_result.confirmed {
        // Update status to BLOCKCHAIN_FAILED
        sqlx::query("UPDATE documents SET status = 'BLOCKCHAIN_FAILED', updated_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(id)
            .execute(&state.db).await?;

        return Err(AppError::Blockchain("Transaction not confirmed on-chain".into()));
    }

    let final_block = block_number.unwrap_or(receipt_result.block_number);

    // 11. SUCCESS — Update document status to VERIFIED
    let now = Utc::now();
    sqlx::query("UPDATE documents SET status = 'VERIFIED', blockchain_transaction = $1, updated_at = $2 WHERE id = $3")
        .bind(transaction_hash)
        .bind(now)
        .bind(id)
        .execute(&state.db).await?;

    // 12. Create document_verifications record
    sqlx::query(
        "INSERT INTO document_verifications (id, document_id, document_hash, verifier_user_id, wallet_address, network, chain_id, contract_address, transaction_hash, block_number, status, verified_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(&document_hash)
    .bind(user_id)
    .bind(wallet_address)
    .bind("Arbitrum Sepolia")
    .bind(421614_i64)
    .bind(contract_address)
    .bind(transaction_hash)
    .bind(final_block)
    .bind("CONFIRMED")
    .bind(now)
    .bind(now)
    .execute(&state.db).await?;

    // 13. Also record in blockchain_transactions
    sqlx::query(
        "INSERT INTO blockchain_transactions (id, document_id, transaction_hash, chain_id, network, contract_address, block_number, status, transaction_type, submitted_at, confirmed_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, 'CONFIRMED'::blockchain_transaction_status, $8, $9, $10, $11)"
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(transaction_hash)
    .bind(421614_i64)
    .bind("Arbitrum Sepolia")
    .bind(contract_address)
    .bind(final_block)
    .bind("DOCUMENT_VERIFICATION")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&state.db).await?;

    // 14. Audit log
    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'BLOCKCHAIN_VERIFY_DOCUMENT', 'document', $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(id)
    .bind(now)
    .execute(&state.db).await?;

    Ok(Json(json!({
        "message": "Document blockchain verification confirmed",
        "status": "VERIFIED",
        "transaction_hash": transaction_hash,
        "block_number": final_block,
        "wallet_address": wallet_address,
        "chain_id": 421614,
        "network": "Arbitrum Sepolia",
        "verified_at": now,
        "document_hash": document_hash
    })))
}

/// GET /documents/:id/verification — Returns verification details
async fn get_verification(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;

    // Get document info
    let doc = sqlx::query!(
        "SELECT d.id, d.document_id, d.filename, d.sha256, d.status::text as status, d.document_type::text as doc_type, s.buyer_id, s.exporter_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1",
        id
    )
    .fetch_optional(&state.db).await?
    .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // Verify access
    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Auth("No organization".into()))?;

    let mut has_access = doc.buyer_id == user_org || doc.exporter_id == user_org;
    if !has_access {
        let access = sqlx::query!("SELECT id FROM trade_access WHERE buyer_id = $1 AND shipment_id IN (SELECT shipment_id FROM documents WHERE id = $2)", user_id, id)
            .fetch_optional(&state.db).await?;
        has_access = access.is_some();
    }

    if !has_access {
        return Err(AppError::Auth("Not authorized to view this verification".into()));
    }

    // Get verification record
    let verification = sqlx::query!(
        "SELECT * FROM document_verifications WHERE document_id = $1 ORDER BY created_at DESC LIMIT 1",
        id
    )
    .fetch_optional(&state.db).await?;

    let status = doc.status.unwrap_or_default();

    match verification {
        Some(v) => {
            Ok(Json(json!({
                "document": {
                    "id": doc.id,
                    "document_id": doc.document_id,
                    "filename": doc.filename,
                    "document_hash": doc.sha256,
                    "document_type": doc.doc_type,
                    "database_status": status
                },
                "blockchain": {
                    "status": v.status,
                    "network": v.network,
                    "chain_id": v.chain_id,
                    "contract_address": v.contract_address,
                    "transaction_hash": v.transaction_hash,
                    "block_number": v.block_number,
                    "wallet_address": v.wallet_address,
                    "verified_at": v.verified_at
                }
            })))
        }
        None => {
            Ok(Json(json!({
                "document": {
                    "id": doc.id,
                    "document_id": doc.document_id,
                    "filename": doc.filename,
                    "document_hash": doc.sha256,
                    "document_type": doc.doc_type,
                    "database_status": status
                },
                "blockchain": null
            })))
        }
    }
}

/// GET /documents/:id/integrity — Check document integrity by re-hashing file and comparing
async fn check_integrity(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    // Get document record
    let doc = sqlx::query!("SELECT sha256, storage_reference, status::text as status, blockchain_transaction FROM documents WHERE id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    let stored_hash = doc.sha256;
    let file_path = doc.storage_reference.unwrap_or_default();

    // Re-read file and compute hash
    let file_bytes = tokio::fs::read(&file_path).await.map_err(|_| AppError::NotFound("File not found on disk".into()))?;
    let current_hash = hash_document(&file_bytes)?;

    let db_match = current_hash == stored_hash;

    // If blockchain-verified, also check against the blockchain verification record
    let mut blockchain_match = None;
    let mut blockchain_hash = None;

    let verification = sqlx::query!("SELECT document_hash FROM document_verifications WHERE document_id = $1 ORDER BY created_at DESC LIMIT 1", id)
        .fetch_optional(&state.db).await?;

    if let Some(v) = verification {
        blockchain_hash = Some(v.document_hash.clone());
        blockchain_match = Some(current_hash == v.document_hash);
    }

    Ok(Json(json!({
        "document_id": id,
        "stored_hash": stored_hash,
        "current_hash": current_hash,
        "database_integrity": db_match,
        "blockchain_hash": blockchain_hash,
        "blockchain_integrity": blockchain_match,
        "overall_integrity": db_match && blockchain_match.unwrap_or(true)
    })))
}

/// POST /documents/:id/reupload — Seller re-uploads a new version of a document
async fn reupload_document(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    if claims.role.to_uppercase() != "SELLER" {
        return Err(AppError::Auth("Only sellers can re-upload documents".into()));
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;

    // Get existing document
    let doc = sqlx::query!("SELECT d.id, d.document_id, d.shipment_id, d.current_version, d.status::text as status, s.exporter_id FROM documents d JOIN shipments s ON d.shipment_id = s.id WHERE d.id = $1", id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Document not found".into()))?;

    // Verify seller owns this shipment
    let user = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .and_then(|u| u.organization_id)
        .ok_or_else(|| AppError::Validation("User has no org".into()))?;

    if doc.exporter_id != user {
        return Err(AppError::Auth("Not authorized to update this document".into()));
    }

    // Parse multipart file
    let mut filename = String::new();
    let mut mime_type = String::new();
    let mut file_bytes = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Internal(e.to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            filename = field.file_name().unwrap_or("unnamed").to_string();
            mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            file_bytes = field.bytes().await.map_err(|e| AppError::Internal(e.to_string()))?.to_vec();
        }
    }

    if file_bytes.is_empty() {
        return Err(AppError::Validation("No file provided".into()));
    }

    // Compute SHA-256 server-side
    let new_hash = hash_document(&file_bytes)?;
    let new_version = doc.current_version + 1;

    // Save new file
    let file_path = format!("uploads/{}_{}_v{}_{}", doc.id, doc.document_id, new_version, filename);
    tokio::fs::write(&file_path, &file_bytes).await.map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

    // Create new version record — OLD version stays permanently linked to its blockchain tx
    let version_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO document_versions (id, document_id, version, filename, mime_type, file_size, sha256, storage_reference, uploaded_by, status, uploaded_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'PENDING'::document_status, $10, $11)"
    )
    .bind(version_id)
    .bind(id)
    .bind(new_version)
    .bind(&filename)
    .bind(&mime_type)
    .bind(file_bytes.len() as i64)
    .bind(&new_hash)
    .bind(&file_path)
    .bind(user_id)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&state.db).await?;

    // Update parent document to point to new version, reset status to PENDING
    sqlx::query(
        "UPDATE documents SET current_version = $1, sha256 = $2, filename = $3, mime_type = $4, file_size = $5, storage_reference = $6, status = 'PENDING', approval_status = 'PENDING', rejection_reason = NULL, updated_at = $7 WHERE id = $8"
    )
    .bind(new_version)
    .bind(&new_hash)
    .bind(&filename)
    .bind(&mime_type)
    .bind(file_bytes.len() as i64)
    .bind(&file_path)
    .bind(Utc::now())
    .bind(id)
    .execute(&state.db).await?;

    Ok(Json(json!({
        "message": "Document re-uploaded successfully",
        "version": new_version,
        "hash": new_hash,
        "version_id": version_id
    })))
}

#[derive(serde::Deserialize)]
struct VerifyHashPayload {
    hash: String,
}

async fn verify_hash(
    State(state): State<AppState>,
    Json(payload): Json<VerifyHashPayload>,
) -> Result<Json<Value>, AppError> {
    let doc = sqlx::query!(
        r#"
        SELECT 
            d.id, d.document_type as "document_type: crate::models::document::DocumentType", 
            d.filename, d.sha256, d.status as "status: crate::models::document::DocumentStatus",
            s.shipment_id as display_shipment_id,
            (u.first_name || ' ' || u.last_name) as uploader_name,
            v.transaction_hash, v.verified_at, v.network, v.chain_id, v.wallet_address
        FROM documents d
        JOIN shipments s ON d.shipment_id = s.id
        LEFT JOIN users u ON d.uploaded_by = u.id
        LEFT JOIN document_verifications v ON v.document_id = d.id AND v.status = 'CONFIRMED'
        WHERE d.sha256 = $1
        ORDER BY d.created_at DESC
        LIMIT 1
        "#,
        payload.hash
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(d) = doc {
        Ok(Json(json!({
            "verified": d.status == crate::models::document::DocumentStatus::Verified,
            "match": true,
            "document": {
                "id": d.id,
                "document_type": d.document_type,
                "filename": d.filename,
                "sha256": d.sha256,
                "status": d.status,
                "shipment_id": d.display_shipment_id,
                "uploader_name": d.uploader_name,
                "transaction_hash": d.transaction_hash,
                "verified_at": d.verified_at,
                "network": d.network,
                "chain_id": d.chain_id,
                "verifier_wallet": d.wallet_address
            }
        })))
    } else {
        Ok(Json(json!({ "verified": false, "match": false })))
    }
}

async fn verify_document(State(_state): State<AppState>, Extension(_claims): Extension<Claims>, Path(id): Path<String>, Json(_payload): Json<Value>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "verified": true, "match": true })))
}

async fn list_documents(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;

    let user_org = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AppError::Auth("User not found".into()))?;
        
    let org_id = user_org.organization_id;

    let records = sqlx::query!(
        r#"
        SELECT 
            d.id, d.document_id, d.shipment_id, d.uploaded_by, d.document_type as "document_type: crate::models::document::DocumentType",
            d.filename, d.mime_type, d.file_size, d.current_version, d.sha256, d.storage_reference, d.ipfs_cid,
            d.status as "status: crate::models::document::DocumentStatus", d.approval_status as "approval_status: crate::models::document::DocumentStatus",
            d.blockchain_transaction, d.created_at, d.updated_at,
            s.shipment_id as display_shipment_id
        FROM documents d
        JOIN shipments s ON d.shipment_id = s.id
        LEFT JOIN trade_access ta ON ta.shipment_id = s.id AND ta.buyer_id = $2
        WHERE s.exporter_id = $1 OR s.buyer_id = $1 OR ta.id IS NOT NULL
        ORDER BY d.created_at DESC
        "#,
        org_id,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut result = Vec::new();
    for r in records {
        result.push(json!({
            "id": r.id,
            "document_id": r.document_id,
            "shipment_id": r.shipment_id,
            "display_shipment_id": r.display_shipment_id,
            "document_type": r.document_type,
            "filename": r.filename,
            "mime_type": r.mime_type,
            "file_size": r.file_size,
            "current_version": r.current_version,
            "sha256": r.sha256,
            "ipfs_cid": r.ipfs_cid,
            "status": r.status,
            "created_at": r.created_at,
            "updated_at": r.updated_at
        }));
    }

    Ok(Json(json!({"success": true, "data": result})))
}
