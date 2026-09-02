use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "document_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentType {
    CommercialInvoice,
    PackingList,
    CertificateOfOrigin,
    QualityCertificate,
    InspectionCertificate,
    InsuranceDocument,
    ShippingDocument,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentStatus {
    Pending,
    Approved,
    Rejected,
    Superseded,
    Revoked,
    Verified,
    BlockchainPending,
    BlockchainFailed,
    BlockchainRejected,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "approval_decision", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    Revoked,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Document {
    pub id: Uuid,
    pub document_id: String,
    pub shipment_id: Uuid,
    pub uploaded_by: Uuid,
    pub document_type: DocumentType,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i32,
    pub current_version: i32,
    pub sha256: String,
    pub storage_reference: Option<String>,
    pub ipfs_cid: Option<String>,
    pub status: DocumentStatus,
    pub approval_status: Option<DocumentStatus>,
    pub blockchain_transaction: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DocumentVersion {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: i32,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i32,
    pub sha256: String,
    pub storage_reference: Option<String>,
    pub ipfs_cid: Option<String>,
    pub uploaded_by: Uuid,
    pub status: DocumentStatus,
    pub uploaded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DocumentApproval {
    pub id: Uuid,
    pub document_id: Uuid,
    pub document_version_id: Uuid,
    pub reviewed_by: Uuid,
    pub decision: ApprovalDecision,
    pub comments: Option<String>,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DocumentVerification {
    pub id: Uuid,
    pub document_id: Uuid,
    pub document_hash: String,
    pub verifier_user_id: Uuid,
    pub wallet_address: String,
    pub network: String,
    pub chain_id: i64,
    pub contract_address: String,
    pub transaction_hash: String,
    pub block_number: Option<i64>,
    pub status: String,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
