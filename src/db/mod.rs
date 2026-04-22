pub mod account_credentials;
pub mod account_sessions;
pub mod accounts;
pub mod api_keys;
pub mod balances;
pub mod conversations;
pub mod image_generations;
pub mod messages;
pub mod migrate;
pub mod models;
pub mod plan_models;
pub mod plans;
pub mod providers;
pub mod proxies;
pub mod public_model_routes;
pub mod public_models;
pub mod transactions;
pub mod usage_logs;
pub mod user_plans;
pub mod users;

// Re-export for convenience
pub use models::{create_model, list_models, list_models_for_plan, update_model};
pub use plan_models::{add_model_to_plan, remove_model_from_plan, set_plan_models};
pub use plans::get_plan;
pub use providers::{create_provider, get_provider, list_providers, update_provider};
pub use usage_logs::{
    count_today_by_api_key, count_today_by_api_key_scope, count_today_by_user,
    count_today_by_user_scope, sum_daily_credits_by_api_key, sum_daily_credits_by_user,
    sum_monthly_credits_by_api_key, sum_monthly_credits_by_user,
};

use sqlx::postgres::PgPoolOptions;

/// Initialize PostgreSQL connection pool
pub async fn init_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
}
