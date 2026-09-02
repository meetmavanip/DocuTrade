pub mod auth;
pub mod shipments;
pub mod documents;
pub mod blockchain;
pub mod tracking;
pub mod trade_codes;
pub mod notifications;

use axum::Router;
use crate::state::AppState;

pub fn api_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/shipments", shipments::routes())
        .nest("/documents", documents::routes())
        .nest("/blockchain", blockchain::routes())
        .nest("/tracking", tracking::routes())
        .nest("/trade-codes", trade_codes::routes())
        .nest("/notifications", notifications::routes())
}
