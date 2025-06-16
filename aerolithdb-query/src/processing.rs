//! # Document Processing and Filtering
//!
//! Core document processing functions including filtering, sorting, and pagination.
//! Provides the fundamental operations for query execution and result processing.

use serde_json::Value;
use std::cmp::Ordering;

/// Document filtering engine for applying query conditions to documents.
///
/// Provides comprehensive document filtering capabilities using MongoDB-style
/// query operators and efficient evaluation algorithms.
pub struct DocumentFilter;

impl DocumentFilter {
    /// Apply a filter to a document and return whether it matches.
    ///
    /// Evaluates the provided filter against the document using MongoDB-style
    /// query operators. Supports complex nested conditions and boolean logic.
    ///
    /// # Arguments
    /// * `document` - The document to evaluate
    /// * `filter` - The filter criteria in MongoDB query format
    ///
    /// # Returns
    /// * `true` if the document matches the filter
    /// * `false` if the document does not match
    ///
    /// # Filter Examples
    /// ```json
    /// // Simple equality
    /// {"status": "active"}
    /// 
    /// // Range queries
    /// {"age": {"$gte": 18, "$lt": 65}}
    /// 
    /// // Array operations
    /// {"tags": {"$in": ["important", "urgent"]}}
    /// 
    /// // Boolean logic
    /// {"$and": [{"status": "active"}, {"type": "premium"}]}
    /// ```
    pub fn matches_filter(document: &Value, filter: &Value) -> bool {
        match filter {
            Value::Object(filter_obj) => {
                for (key, value) in filter_obj {
                    if !Self::matches_field(document, key, value) {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Check if a specific field matches the given filter condition.
    fn matches_field(document: &Value, field: &str, condition: &Value) -> bool {
        match field {
            "$and" => Self::matches_and(document, condition),
            "$or" => Self::matches_or(document, condition),
            "$not" => Self::matches_not(document, condition),
            _ => Self::matches_simple_field(document, field, condition),
        }
    }

    /// Handle $and logical operator.
    fn matches_and(document: &Value, conditions: &Value) -> bool {
        if let Value::Array(conditions_array) = conditions {
            conditions_array.iter().all(|cond| Self::matches_filter(document, cond))
        } else {
            false
        }
    }

    /// Handle $or logical operator.
    fn matches_or(document: &Value, conditions: &Value) -> bool {
        if let Value::Array(conditions_array) = conditions {
            conditions_array.iter().any(|cond| Self::matches_filter(document, cond))
        } else {
            false
        }
    }

    /// Handle $not logical operator.
    fn matches_not(document: &Value, condition: &Value) -> bool {
        !Self::matches_filter(document, condition)
    }

    /// Handle simple field matching with operators.
    fn matches_simple_field(document: &Value, field: &str, condition: &Value) -> bool {
        let field_value = Self::get_nested_field(document, field);
        
        match condition {
            Value::Object(condition_obj) => {
                for (operator, value) in condition_obj {
                    if !Self::matches_operator(&field_value, operator, value) {
                        return false;
                    }
                }
                true
            }
            _ => field_value == *condition,
        }
    }

    /// Get a nested field value from a document using dot notation.
    fn get_nested_field(document: &Value, field: &str) -> Value {
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = document;
        
        for part in parts {
            match current {
                Value::Object(obj) => {
                    if let Some(value) = obj.get(part) {
                        current = value;
                    } else {
                        return Value::Null;
                    }
                }
                _ => return Value::Null,
            }
        }
        
        current.clone()
    }

    /// Apply a comparison operator to field values.
    fn matches_operator(field_value: &Value, operator: &str, operand: &Value) -> bool {
        match operator {
            "$eq" => field_value == operand,
            "$ne" => field_value != operand,
            "$gt" => Self::compare_values(field_value, operand) == Ordering::Greater,
            "$gte" => matches!(Self::compare_values(field_value, operand), Ordering::Greater | Ordering::Equal),
            "$lt" => Self::compare_values(field_value, operand) == Ordering::Less,
            "$lte" => matches!(Self::compare_values(field_value, operand), Ordering::Less | Ordering::Equal),
            "$in" => Self::matches_in(field_value, operand),
            "$nin" => !Self::matches_in(field_value, operand),
            "$regex" => Self::matches_regex(field_value, operand),
            "$exists" => Self::matches_exists(field_value, operand),
            _ => false,
        }
    }

    /// Handle $in operator for array membership.
    fn matches_in(field_value: &Value, array: &Value) -> bool {
        if let Value::Array(arr) = array {
            arr.contains(field_value)
        } else {
            false
        }
    }    /// Handle $regex operator for pattern matching.
    fn matches_regex(field_value: &Value, pattern: &Value) -> bool {
        if let (Value::String(text), Value::String(pattern_str)) = (field_value, pattern) {            // Simple pattern matching - advanced regex support available as enhancement
            text.contains(pattern_str)
        } else {
            false
        }
    }

    /// Handle $exists operator for field presence.
    fn matches_exists(field_value: &Value, should_exist: &Value) -> bool {
        let exists = !field_value.is_null();
        match should_exist {
            Value::Bool(expected) => exists == *expected,
            _ => false,
        }
    }

    /// Compare two JSON values for ordering.
    fn compare_values(a: &Value, b: &Value) -> Ordering {
        match (a, b) {
            (Value::Number(a), Value::Number(b)) => {
                if let (Some(a_f64), Some(b_f64)) = (a.as_f64(), b.as_f64()) {
                    a_f64.partial_cmp(&b_f64).unwrap_or(Ordering::Equal)
                } else {
                    Ordering::Equal
                }
            }
            (Value::String(a), Value::String(b)) => a.cmp(b),
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            _ => Ordering::Equal,
        }
    }

    /// Filter documents based on the provided filter criteria.
    ///
    /// # Arguments
    /// * `documents` - The documents to filter
    /// * `filter` - The filter criteria in MongoDB query format
    ///
    /// # Returns
    /// A vector containing the documents that match the filter criteria
    pub fn filter_documents(documents: Vec<Value>, filter: &Value) -> Vec<Value> {
        documents.into_iter().filter(|doc| Self::matches_filter(doc, filter)).collect()
    }
}

/// Document sorting engine for applying sort orders to document collections.
pub struct DocumentSorter;

impl DocumentSorter {
    /// Sort a collection of documents according to the provided sort specification.
    ///
    /// Supports multi-field sorting with ascending and descending order for each field.
    /// Uses stable sorting to maintain consistent ordering for equal elements.
    ///
    /// # Arguments
    /// * `documents` - Mutable reference to the documents to sort
    /// * `sort_spec` - Sort specification in MongoDB format
    ///
    /// # Sort Specification Format
    /// ```json
    /// // Single field ascending
    /// {"field": 1}
    /// 
    /// // Single field descending  
    /// {"field": -1}
    /// 
    /// // Multi-field sorting
    /// {"priority": -1, "created_at": 1, "name": 1}
    /// ```
    pub fn sort_documents(documents: &mut [Value], sort_spec: &Value) {
        if let Value::Object(sort_obj) = sort_spec {
            documents.sort_by(|a, b| Self::compare_documents(a, b, sort_obj));
        }
    }

    /// Compare two documents according to the sort specification.
    fn compare_documents(a: &Value, b: &Value, sort_spec: &serde_json::Map<String, Value>) -> Ordering {
        for (field, direction) in sort_spec {
            let a_val = DocumentFilter::get_nested_field(a, field);
            let b_val = DocumentFilter::get_nested_field(b, field);
            
            let comparison = DocumentFilter::compare_values(&a_val, &b_val);
            
            if comparison != Ordering::Equal {
                return if Self::is_descending(direction) {
                    comparison.reverse()
                } else {
                    comparison
                };
            }
        }
        Ordering::Equal
    }

    /// Check if the sort direction is descending.
    fn is_descending(direction: &Value) -> bool {
        match direction {
            Value::Number(n) => n.as_i64().unwrap_or(1) < 0,
            Value::String(s) => s == "desc" || s == "descending" || s == "-1",
            _ => false,
        }
    }
}

/// Document pagination engine for efficiently handling large result sets.
pub struct DocumentPaginator;

impl DocumentPaginator {
    /// Apply offset and limit to a document collection.
    ///
    /// Efficiently handles pagination by slicing the document collection
    /// according to the provided offset and limit parameters.
    ///
    /// # Arguments
    /// * `documents` - The complete document collection
    /// * `offset` - Number of documents to skip
    /// * `limit` - Maximum number of documents to return
    ///
    /// # Returns
    /// A vector containing the paginated subset of documents
    pub fn paginate_documents(
        documents: Vec<Value>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Vec<Value> {
        let start = offset.unwrap_or(0);
        let end = if let Some(limit) = limit {
            std::cmp::min(start + limit, documents.len())
        } else {
            documents.len()
        };

        if start >= documents.len() {
            vec![]
        } else {
            documents[start..end].to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_filter_simple_equality() {
        let documents = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
            json!({"name": "Charlie", "age": 35}),
        ];

        let filter = json!({"name": "Alice"});
        let result = DocumentFilter::filter_documents(documents, &filter);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "Alice");
    }

    #[test]
    fn test_document_filter_comparison_operators() {
        let documents = vec![
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
            json!({"name": "Charlie", "age": 35}),
        ];

        // Test greater than
        let filter = json!({"age": {"$gt": 30}});
        let result = DocumentFilter::filter_documents(documents.clone(), &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "Charlie");

        // Test less than
        let filter = json!({"age": {"$lt": 30}});
        let result = DocumentFilter::filter_documents(documents.clone(), &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "Bob");

        // Test greater than or equal
        let filter = json!({"age": {"$gte": 30}});
        let result = DocumentFilter::filter_documents(documents.clone(), &filter);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_document_filter_in_operator() {
        let documents = vec![
            json!({"name": "Alice", "category": "admin"}),
            json!({"name": "Bob", "category": "user"}),
            json!({"name": "Charlie", "category": "moderator"}),
        ];

        let filter = json!({"category": {"$in": ["admin", "moderator"]}});
        let result = DocumentFilter::filter_documents(documents, &filter);
        
        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter()
            .map(|doc| doc["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Charlie"));
    }

    #[test]
    fn test_document_filter_exists_operator() {
        let documents = vec![
            json!({"name": "Alice", "email": "alice@example.com"}),
            json!({"name": "Bob"}),
            json!({"name": "Charlie", "email": "charlie@example.com"}),
        ];

        let filter = json!({"email": {"$exists": true}});
        let result = DocumentFilter::filter_documents(documents.clone(), &filter);
        assert_eq!(result.len(), 2);

        let filter = json!({"email": {"$exists": false}});
        let result = DocumentFilter::filter_documents(documents, &filter);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "Bob");
    }

    #[test]
    fn test_document_filter_nested_fields() {
        let documents = vec![
            json!({"user": {"profile": {"age": 30}}}),
            json!({"user": {"profile": {"age": 25}}}),
            json!({"user": {"profile": {"age": 35}}}),
        ];

        let filter = json!({"user.profile.age": {"$gt": 25}});
        let result = DocumentFilter::filter_documents(documents, &filter);
        
        assert_eq!(result.len(), 2);
    }    #[test]
    fn test_document_sorter_single_field() {
        let mut documents = vec![
            json!({"name": "Charlie", "age": 35}),
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];

        let sort_spec = json!({"age": 1}); // ascending
        DocumentSorter::sort_documents(&mut documents, &sort_spec);
        
        assert_eq!(documents[0]["name"], "Bob");
        assert_eq!(documents[1]["name"], "Alice");
        assert_eq!(documents[2]["name"], "Charlie");

        let sort_spec = json!({"age": -1}); // descending
        DocumentSorter::sort_documents(&mut documents, &sort_spec);
        
        assert_eq!(documents[0]["name"], "Charlie");
        assert_eq!(documents[1]["name"], "Alice");
        assert_eq!(documents[2]["name"], "Bob");
    }    #[test]
    fn test_document_sorter_multiple_fields() {
        let mut documents = vec![
            json!({"department": "Engineering", "name": "Bob"}),
            json!({"department": "Engineering", "name": "Alice"}),
            json!({"department": "Marketing", "name": "Charlie"}),
        ];

        let sort_spec = json!({"department": 1, "name": 1});
        DocumentSorter::sort_documents(&mut documents, &sort_spec);
        
        assert_eq!(documents[0]["name"], "Alice");
        assert_eq!(documents[1]["name"], "Bob");
        assert_eq!(documents[2]["name"], "Charlie");
    }    #[test]
    fn test_document_sorter_string_direction() {
        let mut documents = vec![
            json!({"name": "Charlie", "age": 35}),
            json!({"name": "Alice", "age": 30}),
            json!({"name": "Bob", "age": 25}),
        ];

        let sort_spec = json!({"age": "desc"});
        DocumentSorter::sort_documents(&mut documents, &sort_spec);
        
        assert_eq!(documents[0]["name"], "Charlie");
        assert_eq!(documents[1]["name"], "Alice");
        assert_eq!(documents[2]["name"], "Bob");
    }

    #[test]
    fn test_document_paginator_basic() {
        let documents = vec![
            json!({"id": 1}),
            json!({"id": 2}),
            json!({"id": 3}),
            json!({"id": 4}),
            json!({"id": 5}),
        ];

        // Test with both offset and limit
        let result = DocumentPaginator::paginate_documents(documents.clone(), Some(1), Some(2));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["id"], 2);
        assert_eq!(result[1]["id"], 3);

        // Test with only limit
        let result = DocumentPaginator::paginate_documents(documents.clone(), None, Some(3));
        assert_eq!(result.len(), 3);
        assert_eq!(result[0]["id"], 1);
        assert_eq!(result[2]["id"], 3);

        // Test with only offset
        let result = DocumentPaginator::paginate_documents(documents.clone(), Some(3), None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0]["id"], 4);
        assert_eq!(result[1]["id"], 5);
    }

    #[test]
    fn test_document_paginator_edge_cases() {
        let documents = vec![
            json!({"id": 1}),
            json!({"id": 2}),
        ];

        // Test offset beyond documents length
        let result = DocumentPaginator::paginate_documents(documents.clone(), Some(5), Some(2));
        assert_eq!(result.len(), 0);

        // Test limit larger than remaining documents
        let result = DocumentPaginator::paginate_documents(documents.clone(), Some(1), Some(5));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["id"], 2);

        // Test with empty documents
        let result = DocumentPaginator::paginate_documents(vec![], Some(0), Some(5));
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_value_comparison() {
        // Test number comparisons
        assert!(DocumentFilter::compare_values(&json!(10), &json!(5)) == std::cmp::Ordering::Greater);
        assert!(DocumentFilter::compare_values(&json!(5), &json!(10)) == std::cmp::Ordering::Less);
        assert!(DocumentFilter::compare_values(&json!(5), &json!(5)) == std::cmp::Ordering::Equal);

        // Test string comparisons
        assert!(DocumentFilter::compare_values(&json!("zebra"), &json!("apple")) == std::cmp::Ordering::Greater);
        assert!(DocumentFilter::compare_values(&json!("apple"), &json!("zebra")) == std::cmp::Ordering::Less);
        assert!(DocumentFilter::compare_values(&json!("same"), &json!("same")) == std::cmp::Ordering::Equal);

        // Test boolean comparisons
        assert!(DocumentFilter::compare_values(&json!(true), &json!(false)) == std::cmp::Ordering::Greater);
        assert!(DocumentFilter::compare_values(&json!(false), &json!(true)) == std::cmp::Ordering::Less);
    }

    #[test]
    fn test_nested_field_access() {
        let document = json!({
            "user": {
                "profile": {
                    "contact": {
                        "email": "test@example.com"
                    }
                }
            }
        });

        let result = DocumentFilter::get_nested_field(&document, "user.profile.contact.email");
        assert_eq!(result, json!("test@example.com"));

        let result = DocumentFilter::get_nested_field(&document, "user.profile.nonexistent");
        assert_eq!(result, Value::Null);

        let result = DocumentFilter::get_nested_field(&document, "user");
        assert!(result.is_object());
    }

    #[test]
    fn test_complex_filter_and_sort_combination() {
        let documents = vec![
            json!({"name": "Alice", "age": 30, "department": "Engineering"}),
            json!({"name": "Bob", "age": 25, "department": "Engineering"}),
            json!({"name": "Charlie", "age": 35, "department": "Marketing"}),
            json!({"name": "Diana", "age": 28, "department": "Engineering"}),
        ];

        // Filter for Engineering department and age > 25
        let filter = json!({"department": "Engineering", "age": {"$gt": 25}});        let mut filtered = DocumentFilter::filter_documents(documents, &filter);
        
        assert_eq!(filtered.len(), 2); // Alice and Diana
        
        // Sort by age descending
        let sort_spec = json!({"age": -1});
        DocumentSorter::sort_documents(&mut filtered, &sort_spec);
        
        assert_eq!(filtered[0]["name"], "Alice"); // age 30
        assert_eq!(filtered[1]["name"], "Diana"); // age 28
        
        // Paginate to get only first result
        let paginated = DocumentPaginator::paginate_documents(filtered, Some(0), Some(1));
        
        assert_eq!(paginated.len(), 1);
        assert_eq!(paginated[0]["name"], "Alice");
    }
}
