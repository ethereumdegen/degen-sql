use crate::tiny_safe_string::TinySafeString;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::fmt;
use serde::de::{self, Visitor};

#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone)]
pub struct PaginationData {
    pub page: Option<i64>,       // Current page (1-indexed)
    pub page_size: Option<i64>,  // Number of items per page
    pub sort_by: Option<TinySafeString>, // Column to sort by
    pub sort_dir: Option<ColumnSortDir>, // Sort direction (asc/desc)
}

#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
#[derive(Debug, Clone)]
pub enum ColumnSortDir { 
    Asc,
    Desc
}

// Custom serialization for ColumnSortDir
impl Serialize for ColumnSortDir {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ColumnSortDir::Asc => serializer.serialize_str("asc"),
            ColumnSortDir::Desc => serializer.serialize_str("desc"),
        }
    }
}

// Custom deserialization for ColumnSortDir
impl<'de> Deserialize<'de> for ColumnSortDir {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ColumnSortDirVisitor;
        impl<'de> Visitor<'de> for ColumnSortDirVisitor {
            type Value = ColumnSortDir;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing sort direction: 'asc' or 'desc'")
            }
            fn visit_str<E>(self, value: &str) -> Result<ColumnSortDir, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "asc" => Ok(ColumnSortDir::Asc),
                    "desc" => Ok(ColumnSortDir::Desc),
                    _ => Ok(ColumnSortDir::Desc), // Default to DESC for invalid values
                }
            }
        }
        deserializer.deserialize_str(ColumnSortDirVisitor)
    }
}

// Add Serialize for the whole PaginationData struct
impl Serialize for PaginationData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PaginationData", 4)?;
        state.serialize_field("page", &self.page)?;
        state.serialize_field("page_size", &self.page_size)?;
        state.serialize_field("sort_by", &self.sort_by)?;
        state.serialize_field("sort_dir", &self.sort_dir)?;
        state.end()
    }
}

// Add Deserialize for the whole PaginationData struct
impl<'de> Deserialize<'de> for PaginationData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PaginationDataHelper {
            page: Option<i64>,
            page_size: Option<i64>,
            sort_by: Option<TinySafeString>,
            sort_dir: Option<ColumnSortDir>,
        }
        let helper = PaginationDataHelper::deserialize(deserializer)?;
        
        Ok(PaginationData {
            page: helper.page,
            page_size: helper.page_size,
            sort_by: helper.sort_by,
            sort_dir: helper.sort_dir,
        })
    }
}

impl Default for PaginationData {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(10),
            sort_by: Some(TinySafeString::new("created_at").unwrap()),
            sort_dir: Some(ColumnSortDir::Desc),
        }
    }
}

impl ColumnSortDir {
    // Convert enum to SQL direction
    pub fn to_sql_string(&self) -> &'static str {
        match self {
            ColumnSortDir::Asc => "ASC",
            ColumnSortDir::Desc => "DESC",
        }
    }
}

impl PaginationData {
    // Get SQL limit clause
    pub fn get_limit(&self) -> i64 {
        self.page_size.unwrap_or(10).min(100) // Limit maximum page size to 100
    }
    
    // Get SQL offset clause
    pub fn get_offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1) - 1; // 1-indexed to 0-indexed
        page * self.get_limit()
    }
    
    // Get SQL order by clause
    pub fn get_order_by(&self) -> String {
        let column = match &self.sort_by {
            Some(col) => col.to_sql_string(),
            None => "created_at", // Default column
        };
        
        let direction = match &self.sort_dir {
            Some(dir) => dir.to_sql_string(),
            None => "DESC", // Default direction
        };
        
        format!("{} {}", column, direction)
    }
    
    // Build the SQL pagination part
    pub fn build_query_part(&self) -> String {
        format!(
            "ORDER BY {} LIMIT {} OFFSET {}",
            self.get_order_by(),
            self.get_limit(),
            self.get_offset()
        )
    }
}

#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
#[derive(Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}