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
pub struct InvoiceTemplate {
    pub id: i64,
    pub owner_address: String,
    pub chain_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl InvoiceTemplate {
    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError> {
        Ok(InvoiceTemplate {
            id: row.get("id"),
            owner_address: row.get("owner_address"),
            chain_id: row.get("chain_id"),
            title: row.get("title"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

pub struct InvoiceTemplatesModel;

impl InvoiceTemplatesModel {
    pub async fn find_by_owner(
        &self,
        owner_address: &str,
        chain_id: i64,
        pagination: Option<&PaginationData>,
        psql_db: &Database,
    ) -> Result<Vec<InvoiceTemplate>, PostgresModelError> {
        // Create where params
        let mut where_params: BTreeMap<TinySafeString, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert(TinySafeString::new("owner_address").unwrap(), Arc::new(owner_address.to_string()));
        where_params.insert(TinySafeString::new("chain_id").unwrap(), Arc::new(chain_id));
        
        // Build SQL query
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "invoice_templates".to_string(),
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
        let mut templates = Vec::new();
        for row in rows {
            match InvoiceTemplate::from_row(&row) {
                Ok(template) => templates.push(template),
                Err(e) => {
                    eprintln!("Error parsing invoice template row: {}", e);
                    // Continue to next row
                }
            }
        }
        
        Ok(templates)
    }
    
    pub async fn count_by_owner(
        &self,
        owner_address: &str, 
        chain_id: i64,
        psql_db: &Database,
    ) -> Result<i64, PostgresModelError> {
        let mut where_params: BTreeMap<TinySafeString, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert(TinySafeString::new("owner_address").unwrap(), Arc::new(owner_address.to_string()));
        where_params.insert(TinySafeString::new("chain_id").unwrap(), Arc::new(chain_id));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "invoice_templates".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        let row = psql_db.query_one(&query, built_params).await?;
        let count: i64 = row.get(0);
        
        Ok(count)
    }
}