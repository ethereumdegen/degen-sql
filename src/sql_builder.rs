
use std::sync::Arc;
use std::collections::BTreeMap;

use tokio_postgres::types::ToSql;
use crate::pagination::PaginationData;
use crate::tiny_safe_string::TinySafeString;


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
 
	pub where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)>, 
    
	pub order: Option<(TinySafeString,OrderingDirection)> , 
 
 
 
	pub limit: Option< u32 >, 
	
	// Optional pagination that overrides order, limit and offset when provided
	pub pagination: Option<PaginationData>,
}

impl SqlBuilder {

        // Create a new instance with default values
            pub fn new(statement_base: SqlStatementBase, table_name: impl Into<String>) -> Self {
                SqlBuilder {
                    statement_base,
                    table_name: table_name.into(),
                    where_params: BTreeMap::new(),
                    order: None,
                    limit: None,
                    pagination: None,
                }
            }
            
            // Add a where condition with equality comparison
            pub fn where_eq(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static
         ) -> Self {
                self.where_params.insert(key.into(), (ComparisonType::EQ, Arc::new(value) as Arc<dyn ToSql + Sync>));
                self
            }
            
           // Add a where condition with less than comparison
            pub fn where_lt(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static
         ) -> Self {
                self.where_params.insert(key.into(), (ComparisonType::LT, Arc::new(value) as Arc<dyn ToSql + Sync>));
                self
           }
           
           // Add a where condition with greater than comparison
           pub fn where_gt(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static
         ) -> Self {
               self.where_params.insert(key.into(), (ComparisonType::GT, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a where condition with less than or equal comparison
           pub fn where_lte(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static) -> Self {
               self.where_params.insert(key.into(), (ComparisonType::LTE, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a where condition with greater than or equal comparison
           pub fn where_gte(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static) -> Self {
               self.where_params.insert(key.into(), (ComparisonType::GTE, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a where condition with LIKE comparison
           pub fn where_like(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static) -> Self {
               self.where_params.insert(key.into(), (ComparisonType::LIKE, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a where condition with IN comparison
           pub fn where_in(mut self, key: impl Into<TinySafeString>, value: impl ToSql + Sync + 'static
         ) -> Self {
               self.where_params.insert(key.into(), (ComparisonType::IN, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a where condition with IS NULL comparison
           pub fn where_null(mut self, key: impl Into<TinySafeString>) -> Self {
               // The value doesn't matter for NULL comparison, just using a dummy value
               self.where_params.insert(key.into(), (ComparisonType::NULL, Arc::new(0_i32) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Add a generic where condition with custom comparison
           pub fn where_custom(mut self, key: impl Into<TinySafeString>, comparison_type: ComparisonType, value: impl ToSql + Sync + 'static) -> Self {
               self.where_params.insert(key.into(), (comparison_type, Arc::new(value) as Arc<dyn ToSql + Sync>));
               self
           }
           
           // Set the ORDER BY clause
           pub fn order_by(mut self, column: impl Into<TinySafeString>, direction: OrderingDirection) -> Self {
               self.order = Some((column.into(), direction));
               self
           }
           
           // Set the LIMIT clause
           pub fn limit(mut self, limit: u32) -> Self {
               self.limit = Some(limit);
               self
           }
           
           // Helper method to set pagination
           pub fn with_pagination(mut self, pagination: PaginationData) -> Self {
               self.pagination = Some(pagination);
               self
           }




    pub fn build(&self) -> (String , Vec<Arc<dyn ToSql + Sync>>  ) {
        let mut query = format!("{} FROM {}", self.statement_base.build(), self.table_name);
        let mut conditions = Vec::new();
       

           let mut params: Vec<Arc<dyn ToSql + Sync>> = Vec::new();
        
        // WHERE conditions
        for (key, (comparison_type, param)) in &self.where_params {
            params.push(Arc::clone(param)); // Clone Arc reference
            
            let operator = comparison_type.to_operator();
            if *comparison_type == ComparisonType::IN {
                conditions.push(format!("{} {} (${})", key, operator, params.len()));
            } else if *comparison_type == ComparisonType::NULL {
                conditions.push(format!("{} {}", key, operator));
                // Pop the last parameter as NULL doesn't need a parameter
                params.pop();
            } else {
                conditions.push(format!("{} {} ${}", key, operator, params.len()));
            }
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
}




#[derive(PartialEq,Default)]
pub enum ComparisonType {
    #[default]
    EQ,
    LT,
    GT,
    LTE,
    GTE,
    LIKE,
    IN,
    NULL
}

impl ComparisonType {
    pub fn to_operator(&self) -> &str {
        match self {
            Self::EQ => "=",
            Self::LT => "<",
            Self::GT => ">",
            Self::LTE => "<=",
            Self::GTE => ">=",
            Self::LIKE => "LIKE",
            Self::IN => "IN",
            Self::NULL => "IS NULL",
        }
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
   
    #[test]
    fn test_sql_builder() {
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("chain_id".into(), (ComparisonType::EQ, Arc::new(1_i64) as Arc<dyn ToSql + Sync>));
        where_params.insert("status".into(), (ComparisonType::EQ, Arc::new("active".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "teller_bids".into(),
            where_params,
            order: Some(("created_at".into(), OrderingDirection::DESC)),
            limit: Some(10),
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "SELECT * FROM teller_bids WHERE chain_id = $1 AND status = $2 ORDER BY created_at DESC LIMIT 10"
        );
        assert_eq!(params.len(), 2);
    }
    
    #[test]
    fn test_sql_builder_with_different_comparison_types() {
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("amount".into(), (ComparisonType::GT, Arc::new(1000_i64) as Arc<dyn ToSql + Sync>));
        where_params.insert("created_at".into(), (ComparisonType::LTE, Arc::new("2023-01-01".to_string()) as Arc<dyn ToSql + Sync>));
        where_params.insert("name".into(), (ComparisonType::LIKE, Arc::new("%test%".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "transactions".into(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "SELECT * FROM transactions WHERE amount > $1 AND created_at <= $2 AND name LIKE $3"
        );
        assert_eq!(params.len(), 3);
    }
    
    #[test]
    fn test_sql_builder_with_null_comparison() {
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        // The parameter value doesn't matter for NULL comparison, but we need to provide something
        where_params.insert("deleted_at".into(), (ComparisonType::NULL, Arc::new(0_i32) as Arc<dyn ToSql + Sync>));
        where_params.insert("status".into(), (ComparisonType::EQ, Arc::new("active".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "users".into(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "SELECT * FROM users WHERE deleted_at IS NULL AND status = $1"
        );
        // Only one parameter because NULL doesn't need a parameter
        assert_eq!(params.len(), 1);
    }
    
    #[test]
    fn test_sql_builder_with_in_operator() {
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        // For an IN condition, you'd typically pass an array value
        where_params.insert("status".into(), (ComparisonType::IN, Arc::new("(1, 2, 3)".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectCountAll,
            table_name: "orders".into(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "SELECT COUNT(*) FROM orders WHERE status IN ($1)"
        );
        assert_eq!(params.len(), 1);
    }
    
    #[test]
    fn test_sql_builder_with_pagination() {
        let pagination = PaginationData {
            page: 2,
            items_per_page: 20,
            order_column: "created_at".into(),
            order_direction: OrderingDirection::DESC,
        };
        
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("active".into(), (ComparisonType::EQ, Arc::new(true) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "products".into(),
            where_params,
            order: Some(("id".into(), OrderingDirection::ASC)), // This should be overridden by pagination
            limit: Some(50), // This should be overridden by pagination
            pagination: Some(pagination),
        };
        
        let (query, params) = sql_builder.build();
        // The exact query depends on how the PaginationData.build_query_part() method is implemented
        assert!(query.contains("FROM products WHERE active = $1"));
        assert_eq!(params.len(), 1);
    }
    
    #[test]
    fn test_delete_statement() {
        let mut where_params: BTreeMap<TinySafeString, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("id".into(), (ComparisonType::EQ, Arc::new(42_i64) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::Delete,
            table_name: "logs".into(),
            where_params,
            order: None,
            limit: None,
            pagination: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "DELETE FROM logs WHERE id = $1"
        );
        assert_eq!(params.len(), 1);
    }
}