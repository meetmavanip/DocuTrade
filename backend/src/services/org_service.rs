use sqlx::PgPool;
use uuid::Uuid;
use crate::models::organization::{Organization, OrganizationType};
use crate::errors::AppError;

pub async fn create_organization(
    pool: &PgPool,
    name: &str,
    org_type: OrganizationType,
    country: &str,
) -> Result<Organization, AppError> {
    let org = sqlx::query_as!(
        Organization,
        r#"
        INSERT INTO organizations (name, organization_type, country)
        VALUES ($1, $2, $3)
        RETURNING id, name, legal_name, organization_type AS "organization_type: OrganizationType", 
                  registration_number, tax_id, country, address, city, state, postal_code, 
                  email, phone, website, wallet_address, is_verified, verified_at, verified_by, 
                  created_at, updated_at
        "#,
        name,
        org_type as OrganizationType,
        country
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(org)
}

pub async fn get_organizations(pool: &PgPool) -> Result<Vec<Organization>, AppError> {
    let orgs = sqlx::query_as!(
        Organization,
        r#"
        SELECT id, name, legal_name, organization_type AS "organization_type: OrganizationType", 
               registration_number, tax_id, country, address, city, state, postal_code, 
               email, phone, website, wallet_address, is_verified, verified_at, verified_by, 
               created_at, updated_at
        FROM organizations
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(orgs)
}
