use std::sync::Arc;
use std::collections::BTreeMap;
use tokio_postgres::types::ToSql;
use tokio_postgres::Row;
use crate::sql_builder::{SqlBuilder, SqlStatementBase, OrderingDirection};
use super::model::PostgresModelError;
use deadpool_postgres::Client as Database;

#[derive(Debug, Clone)]
pub struct PremiumStatus {
    pub id: i64,
    pub user_address: String,
    pub chain_id: i64,
    pub is_premium: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PremiumStatus {
    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError> {
        Ok(PremiumStatus {
            id: row.get("id"),
            user_address: row.get("user_address"),
            chain_id: row.get("chain_id"),
            is_premium: row.get("is_premium"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

pub struct PremiumStatusModel;

impl PremiumStatusModel {
    pub async fn get_premium_status(
        &self,
        user_address: &str,
        chain_id: i64,
        psql_db: &Database,
    ) -> Result<Option<PremiumStatus>, PostgresModelError> {
        // Create where params
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("user_address".to_string(), Arc::new(user_address.to_string()));
        where_params.insert("chain_id".to_string(), Arc::new(chain_id));
        
        // Build SQL query
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "premium_status".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: Some(1), // Only need the most recent status
            pagination: None,
        };
        
        // Get query and params
        let (query, params) = sql_builder.build();
        let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        // Execute query
        let rows = psql_db.query(&query, built_params).await?;
        
        // Return the first row if it exists
        if rows.is_empty() {
            return Ok(None);
        }
        
        // Parse the row
        match PremiumStatus::from_row(&rows[0]) {
            Ok(premium_status) => Ok(Some(premium_status)),
            Err(e) => {
                eprintln!("Error parsing premium status row: {}", e);
                Ok(None)
            }
        }
    }
    
    pub async fn is_premium(
        &self,
        user_address: &str,
        chain_id: i64,
        psql_db: &Database,
    ) -> Result<bool, PostgresModelError> {
        // Get the premium status
        let premium_status = self.get_premium_status(user_address, chain_id, psql_db).await?;
        
        // Check if premium and not expired
        match premium_status {
            Some(status) => {
                if !status.is_premium {
                    return Ok(false);
                }
                
                // Check if premium has expired
                if let Some(expires_at) = status.expires_at {
                    let now = chrono::Utc::now();
                    Ok(now < expires_at)
                } else {
                    // No expiration date means premium forever
                    Ok(true)
                }
            },
            None => Ok(false), // No premium status found
        }
    }
}