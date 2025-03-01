use std::sync::Arc;
use std::collections::BTreeMap;
use tokio_postgres::types::ToSql;
use tokio_postgres::Row;
use crate::sql_builder::{SqlBuilder, SqlStatementBase, OrderingDirection};
use crate::pagination::PaginationData;
use super::model::PostgresModelError;
use deadpool_postgres::Client as Database;

#[derive(Debug, Clone)]
pub struct Payment {
    pub id: i64,
    pub invoice_id: i64,
    pub amount: String,
    pub currency: String,
    pub payer_address: String,
    pub recipient_address: String,
    pub chain_id: i64,
    pub tx_hash: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Payment {
    pub fn from_row(row: &Row) -> Result<Self, PostgresModelError> {
        Ok(Payment {
            id: row.get("id"),
            invoice_id: row.get("invoice_id"),
            amount: row.get("amount"),
            currency: row.get("currency"),
            payer_address: row.get("payer_address"),
            recipient_address: row.get("recipient_address"),
            chain_id: row.get("chain_id"),
            tx_hash: row.get("tx_hash"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

pub struct PaymentsModel;

impl PaymentsModel {
    pub async fn find_by_invoice_id(
        &self,
        invoice_id: i64,
        pagination: Option<&PaginationData>,
        psql_db: &Database,
    ) -> Result<Vec<Payment>, PostgresModelError> {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("invoice_id".to_string(), Arc::new(invoice_id));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "payments".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: None,
            pagination: pagination.cloned(),
        };
        
        let (query, params) = sql_builder.build();
        let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        let rows = psql_db.query(&query, built_params).await?;
        
        let mut payments = Vec::new();
        for row in rows {
            match Payment::from_row(&row) {
                Ok(payment) => payments.push(payment),
                Err(e) => {
                    eprintln!("Error parsing payment row: {}", e);
                    // Continue to next row
                }
            }
        }
        
        Ok(payments)
    }
    
    pub async fn find_by_recipient(
        &self,
        recipient_address: &str,
        chain_id: i64, 
        pagination: Option<&PaginationData>,
        psql_db: &Database,
    ) -> Result<Vec<Payment>, PostgresModelError> {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("recipient_address".to_string(), Arc::new(recipient_address.to_string()));
        where_params.insert("chain_id".to_string(), Arc::new(chain_id));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "payments".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: None,
            pagination: pagination.cloned(),
        };
        
        let (query, params) = sql_builder.build();
        let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();
        
        let rows = psql_db.query(&query, built_params).await?;
        
        let mut payments = Vec::new();
        for row in rows {
            match Payment::from_row(&row) {
                Ok(payment) => payments.push(payment),
                Err(e) => {
                    eprintln!("Error parsing payment row: {}", e);
                }
            }
        }
        
        Ok(payments)
    }
    
    pub async fn count_by_invoice_id(
        &self,
        invoice_id: i64,
        psql_db: &Database,
    ) -> Result<i64, PostgresModelError> {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("invoice_id".to_string(), Arc::new(invoice_id));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "payments".to_string(),
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