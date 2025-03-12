

use std::sync::Arc;
use std::collections::BTreeMap;

use tokio_postgres::types::ToSql;

pub struct SqlBuilder {

	pub statement_base: SqlStatementBase,
	pub table_name : String, 
	pub where_params: BTreeMap<String, (ComparisonType, Arc<dyn ToSql + Sync>)>, 
    
	pub order: Option<(String,OrderingDirection)> , 

	pub limit: Option< u32 >, 


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

        // ORDER BY clause
        if let Some((column, direction)) = &self.order {
            query.push_str(&format!(" ORDER BY {} {}", column, direction.build()));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
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
}

impl SqlStatementBase {

	pub fn build(&self) -> String {

		match self {

			Self::SelectAll => "SELECT *" 

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
        let mut where_params: BTreeMap<String, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("chain_id".to_string(), (ComparisonType::EQ, Arc::new(1_i64) as Arc<dyn ToSql + Sync>));
        where_params.insert("status".to_string(), (ComparisonType::EQ, Arc::new("active".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "teller_bids".to_string(),
            where_params,
            order: Some(("created_at".to_string(), OrderingDirection::DESC)),
            limit: Some(10),
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
        let mut where_params: BTreeMap<String, (ComparisonType, Arc<dyn ToSql + Sync>)> = BTreeMap::new();
        where_params.insert("amount".to_string(), (ComparisonType::GT, Arc::new(1000_i64) as Arc<dyn ToSql + Sync>));
        where_params.insert("created_at".to_string(), (ComparisonType::LTE, Arc::new("2023-01-01".to_string()) as Arc<dyn ToSql + Sync>));
        where_params.insert("name".to_string(), (ComparisonType::LIKE, Arc::new("%test%".to_string()) as Arc<dyn ToSql + Sync>));
        
        let sql_builder = SqlBuilder {
            statement_base: SqlStatementBase::SelectAll,
            table_name: "transactions".to_string(),
            where_params,
            order: None,
            limit: None,
        };
        
        let (query, params) = sql_builder.build();
        assert_eq!(
            query,
            "SELECT * FROM transactions WHERE amount > $1 AND created_at <= $2 AND name LIKE $3"
        );
        assert_eq!(params.len(), 3);
    }
}