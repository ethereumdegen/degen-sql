use std::sync::Arc;
use std::collections::BTreeMap;
use tokio_postgres::types::ToSql;
use tokio_postgres::Row;
use crate::sql_builder::{SqlBuilder, SqlStatementBase, OrderingDirection};
use crate::pagination::PaginationData;
use crate::tiny_safe_string::TinySafeString;
use super::model::PostgresModelError;
use deadpool_postgres::Client as Database;

#[derive(Debug, Clone)]
pub struct WebhookUrl {
    pub id: i64,
    pub owner_address: String,
    pub chain_id: i64,
    pub url: String,
    pub is_active: bool,
    pub last_notified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl WebhookUrl {
    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError> {
        Ok(WebhookUrl {
            id: row.get("id"),
            owner_address: row.get("owner_address"),
            chain_id: row.get("chain_id"),
            url: row.get("url"),
            is_active: row.get("is_active"),
            last_notified_at: row.get("last_notified_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

pub struct WebhookUrlsModel;

impl WebhookUrlsModel {
    pub async fn find_by_owner_address(
        &self,
        owner_address: &str,
        chain_id: i64,
        pagination: Option<&PaginationData>,
        psql_db: &Database,
    ) -> Result<Vec<WebhookUrl>, PostgresModelError> {
        // Create where params
        let mut where_params: BTreeMap<TinySafeString, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert(TinySafeString::new("owner_address").unwrap(), Arc::new(owner_address.to_string()));
        where_params.insert(TinySafeString::new("chain_id").unwrap(), Arc::new(chain_id));
        
        // Build SQL query
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "webhook_urls".to_string(),
            where_params,
            order: Some((TinySafeString::new("created_at").unwrap(), OrderingDirection::DESC)),
            limit: None,
            pagination: pagination.cloned(),
        };
        
        // Get query and params
        let (query, params) = sql_builder.build();
        let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        // Execute query
        let rows = psql_db.query(&query, built_params).await?;
        
        // Parse results
        let mut webhook_urls = Vec::new();
        for row in rows {
            match WebhookUrl::from_row(&row) {
                Ok(webhook_url) => webhook_urls.push(webhook_url),
                Err(e) => {
                    eprintln!("Error parsing webhook URL row: {}", e);
                    // Continue to next row
                }
            }
        }
        
        Ok(webhook_urls)
    }
    
    pub async fn delete_by_id(
        &self,
        id: i64,
        owner_address: &str, // For verification that the owner is deleting their own webhook
        chain_id: i64,
        psql_db: &Database,
    ) -> Result<bool, PostgresModelError> {
        // First verify the webhook exists and belongs to the owner
        let mut where_params: BTreeMap<TinySafeString, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert(TinySafeString::new("id").unwrap(), Arc::new(id));
        where_params.insert(TinySafeString::new("owner_address").unwrap(), Arc::new(owner_address.to_string()));
        where_params.insert(TinySafeString::new("chain_id").unwrap(), Arc::new(chain_id));
        
        let count_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "webhook_urls".to_string(),
            where_params: where_params.clone(),
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (count_query, count_params) = count_builder.build();
        let built_count_params = &count_params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        let row = psql_db.query_one(&count_query, built_count_params).await?;
        let count: i64 = row.get(0);
        
        if count == 0 {
            // Webhook doesn't exist or doesn't belong to the owner
            return Ok(false);
        }
        
        // Now delete the webhook
        let delete_builder = SqlBuilder {
            statement_base: SqlStatementBase::Delete,
            table_name: "webhook_urls".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (delete_query, delete_params) = delete_builder.build();
        let built_delete_params = &delete_params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        let result = psql_db.execute(&delete_query, built_delete_params).await?;
        
        Ok(result > 0)
    }
}