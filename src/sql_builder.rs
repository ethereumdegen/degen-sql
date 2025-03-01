
use std::sync::Arc;
use std::collections::BTreeMap;

use tokio_postgres::types::ToSql;
use crate::pagination::PaginationData;


/*








   let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("owner_address".to_string(), Arc::new(domain_address));
        where_params.insert("chain_id".to_string(), Arc::new(chain_id));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "invoices".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: None,
            pagination: pagination.cloned(),
        };
        
        // Build the SQL query and parameters
        let (query, params) = sql_builder.build();
        

         let built_params = &params.iter().map(|x| &**x).collect::<Vec<_>>();

        // Execute the query
        let rows = psql_db.query(&query, &built_params).await?;

        let mut invoices = Vec::new();
        for row in rows {
            match Invoice::from_row(&row) {
                Ok(invoice) => invoices.push(invoice),
                Err(e) => {
                    eprintln!("Error parsing invoice row: {}", e);
                    // Continue to next row instead of failing entirely
                }
            }
        }

        Ok(invoices)




*/



pub struct SqlBuilder {
	pub statement_base: SqlStatementBase,
	pub table_name : String, 
	pub where_params: BTreeMap<String, Arc<dyn ToSql + Sync> > , 

	pub order: Option<(String,OrderingDirection)> , 

	pub limit: Option< u32 >, 
	
	// Optional pagination that overrides order, limit and offset when provided
	pub pagination: Option<PaginationData>,
}

impl SqlBuilder {
    pub fn build(&self) -> (String , Vec<Arc<dyn ToSql + Sync>>  ) {
        let mut query = format!("{} FROM {}", self.statement_base.build(), self.table_name);
        let mut conditions = Vec::new();
        let mut params: Vec<Arc<dyn ToSql + Sync>> = Vec::new();

        // WHERE conditions
        for (key, param) in &self.where_params {
            params.push(Arc::clone(param)); // Clone Arc reference
            conditions.push(format!("{} = ${}", key, params.len()));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        // Use pagination if provided, otherwise fall back to manual order and limit
        if let Some(pagination) = &self.pagination {
            // Append the pagination query part (includes ORDER BY, LIMIT, and OFFSET)
            query.push_str(&format!(" {}", pagination.build_query_part()));
        } else {
            // ORDER BY clause
            if let Some((column, direction)) = &self.order {
                query.push_str(&format!(" ORDER BY {} {}", column, direction.build()));
            }

            // LIMIT clause
            if let Some(limit) = self.limit {
                query.push_str(&format!(" LIMIT {}", limit));
            }
        }

        ( query , params) 
    }
    
    // Helper method to set pagination
    pub fn with_pagination(mut self, pagination: PaginationData) -> Self {
        self.pagination = Some(pagination);
        self
    }
}



pub enum SqlStatementBase {
	SelectAll,
    SelectCountAll,
    Delete
}

impl SqlStatementBase {

	pub fn build(&self) -> String {

		match self {

			Self::SelectAll => "SELECT *" ,
            Self::SelectCountAll => "SELECT COUNT(*)" ,
            Self::Delete => "DELETE"

		}.to_string() 
	}

}

pub enum OrderingDirection {

	DESC,
	ASC 
}


impl OrderingDirection {

	pub fn build(&self) -> String {

		match self {

			Self::DESC => "DESC" ,
			Self::ASC => "ASC" 

		}.to_string() 
	}

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use crate::pagination::{PaginationData, ColumnSortDir};
    use crate::tiny_safe_string::TinySafeString;

    #[test]
    fn test_sql_builder() {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("chain_id".to_string(), Arc::new(1_i64));
        where_params.insert("status".to_string(), Arc::new("active".to_string()));

        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "teller_bids".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: Some(10),
            pagination: None,
        };

        let (query, params) = sql_builder.build();

        assert_eq!(
            query,
            "SELECT * FROM teller_bids WHERE chain_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT 10"
        );

        assert_eq!(
            params.len(),
            2
        );
    }
    
    #[test]
    fn test_sql_builder_with_pagination() {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("chain_id".to_string(), Arc::new(1_i64));
        
        let mut pagination = PaginationData::default();
        pagination.page = Some(2);
        pagination.page_size = Some(20);
        pagination.sort_by = Some(TinySafeString::new("updated_at").unwrap());
        pagination.sort_dir = Some(ColumnSortDir::Asc);
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "teller_bids".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)), // Should be ignored
            limit: Some(10), // Should be ignored
            pagination: Some(pagination),
        };

        let (query, params) = sql_builder.build();

        assert_eq!(
            query,
            "SELECT * FROM teller_bids WHERE chain_id = $1 ORDER BY updated_at ASC LIMIT 20 OFFSET 20"
        );

        assert_eq!(
            params.len(),
            1
        );
    }
    
    #[test]
    fn test_sql_builder_with_pagination_method() {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("status".to_string(), Arc::new("pending".to_string()));
        
        let mut pagination = PaginationData::default();
        pagination.page = Some(3);
        pagination.page_size = Some(15);
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "orders".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        }.with_pagination(pagination);

        let (query, params) = sql_builder.build();

        assert_eq!(
            query,
            "SELECT * FROM orders WHERE status = $1 ORDER BY created_at DESC LIMIT 15 OFFSET 30"
        );

        assert_eq!(
            params.len(),
            1
        );
    }
    
    // Tests for the example queries in delete_by_apikey function
    #[test]
    fn test_sql_builder_count_query() {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("apikey".to_string(), Arc::new("test-api-key".to_string()));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "api_keys".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        
        assert_eq!(
            query,
            "SELECT COUNT(*) FROM api_keys WHERE apikey = $1"
        );
        
        assert_eq!(
            params.len(),
            1
        );
    }
    
    #[test]
    fn test_sql_builder_delete_query() {
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("apikey".to_string(), Arc::new("test-api-key".to_string()));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::Delete,
            table_name: "api_keys".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        
        assert_eq!(
            query,
            "DELETE FROM api_keys WHERE apikey = $1"
        );
        
        assert_eq!(
            params.len(),
            1
        );
    }
    
    #[test]
    fn test_delete_by_apikey_example() {
        // This test shows how to build both queries from the delete_by_apikey example
        
        // First query: "SELECT COUNT(*) FROM api_keys WHERE apikey = $1;"
        let apikey = "example-api-key";
        let mut where_params: BTreeMap<String, Arc<dyn ToSql + Sync>> = BTreeMap::new();
        where_params.insert("apikey".to_string(), Arc::new(apikey.to_string()));
        
        let count_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "api_keys".to_string(),
            where_params: where_params.clone(),
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (count_query, _count_params) = count_builder.build();
        
        assert_eq!(
            count_query,
            "SELECT COUNT(*) FROM api_keys WHERE apikey = $1"
        );
        
        // Second query: "DELETE FROM api_keys WHERE apikey = $1;"
        let delete_builder = SqlBuilder {
            statement_base: SqlStatementBase::Delete,
            table_name: "api_keys".to_string(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (delete_query, _delete_params) = delete_builder.build();
        
        assert_eq!(
            delete_query,
            "DELETE FROM api_keys WHERE apikey = $1"
        );
        
        // Example of how these might be used (this doesn't execute, just shows the pattern)
        /*
        async fn delete_by_apikey_example(
            apikey: &str,
            psql_db: &Database,
        ) -> Result<bool, PostgresModelError> {
            // First verify the API key exists
            let count_builder = SqlBuilder {
                statement_base: SqlStatementBase::SelectCountAll,
                table_name: "api_keys".to_string(),
                where_params: {
                    let mut map = BTreeMap::new();
                    map.insert("apikey".to_string(), Arc::new(apikey.to_string()));
                    map
                },
                order: None,
                limit: None,
                pagination: None,
            };
            
            let (count_query, count_params) = count_builder.build();
            let check_result = psql_db.query_one(&count_query, &count_params).await?;
            let count: i64 = check_result.get(0);
            
            if count == 0 {
                return Ok(false);
            }
            
            // Now delete the API key
            let delete_builder = SqlBuilder {
                statement_base: SqlStatementBase::Delete,
                table_name: "api_keys".to_string(),
                where_params: {
                    let mut map = BTreeMap::new();
                    map.insert("apikey".to_string(), Arc::new(apikey.to_string()));
                    map
                },
                order: None,
                limit: None,
                pagination: None,
            };
            
            let (delete_query, delete_params) = delete_builder.build();
            let result = psql_db.execute(&delete_query, &delete_params).await;
            
            match result {
                Ok(rows_affected) => Ok(rows_affected > 0),
                Err(e) => Err(e.into()),
            }
        }
        */
    }
}
