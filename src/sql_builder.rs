
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
    pub fn build(&self) -> (String , Vec<Arc<dyn ToSql + Sync>>  ) {
        let mut query = format!("{} FROM {}", self.statement_base.build(), self.table_name);
        let mut conditions = Vec::new();
       

           let mut params: Vec<Arc<dyn ToSql + Sync>> = Vec::new();
        
        // WHERE conditions
        for (key, (comparison_type, param)) in &self.where_params {
            params.push(Arc::clone(param)); // Clone Arc reference
            
            let operator = comparison_type.to_operator();
            if *comparison_type == ComparisonType::IN {
                conditions.push(format!("{} {} (${}}})", key, operator, params.len()));
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
    
    // Helper method to set pagination
    pub fn with_pagination(mut self, pagination: PaginationData) -> Self {
        self.pagination = Some(pagination);
        self
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
}