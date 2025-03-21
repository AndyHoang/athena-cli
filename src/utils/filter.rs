/// Utility functions for filtering collections based on patterns
pub fn matches_pattern<T: AsRef<str>>(value: T, pattern: &str) -> bool {
    let value = value.as_ref();

    // Simple wildcard matching
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();

        // Handle prefix matching (pattern ends with *)
        if pattern.ends_with('*') && parts.len() == 2 {
            return value.starts_with(parts[0]);
        }

        // Handle suffix matching (pattern starts with *)
        if pattern.starts_with('*') && parts.len() == 2 {
            return value.ends_with(parts[1]);
        }

        // Handle contains matching (pattern is *text*)
        if pattern.starts_with('*') && pattern.ends_with('*') && parts.len() == 3 {
            return value.contains(parts[1]);
        }
    } else {
        // Default to substring matching instead of exact matching
        return value.to_lowercase().contains(&pattern.to_lowercase());
    }

    // Exact matching (only reached if none of the wildcard patterns matched)
    value == pattern
}

/// Filter a collection of items based on a pattern
pub fn filter_items<'a, T, F>(items: &'a [T], pattern: Option<&str>, extractor: F) -> Vec<&'a T>
where
    F: Fn(&T) -> &str,
{
    match pattern {
        Some(pattern) => items
            .iter()
            .filter(|item| matches_pattern(extractor(item), pattern))
            .collect(),
        None => items.iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a simple struct to test filtering
    #[derive(PartialEq, Debug)]
    struct TestItem {
        name: String,
        category: String,
        count: i32,
    }

    #[test]
    fn test_matches_pattern() {
        // Test exact match
        assert!(matches_pattern("hello", "hello"));

        // Test substring match (should match if we updated the function)
        assert!(matches_pattern("hello world", "hello"));

        // Test case insensitivity
        assert!(matches_pattern("Hello World", "hello"));

        // Test wildcard patterns
        assert!(matches_pattern("hello world", "hello*"));
        assert!(matches_pattern("hello world", "*world"));
        assert!(matches_pattern("hello world", "*lo wor*"));

        // Test non-matches
        assert!(!matches_pattern("hello", "world"));
        assert!(!matches_pattern("hello", "hello world"));
    }

    #[test]
    fn test_filter_items() {
        // Create a test vector
        let items = vec![
            TestItem {
                name: "Table1".to_string(),
                category: "Data".to_string(),
                count: 10,
            },
            TestItem {
                name: "UserEvents".to_string(),
                category: "Events".to_string(),
                count: 20,
            },
            TestItem {
                name: "EventLog".to_string(),
                category: "Events".to_string(),
                count: 30,
            },
            TestItem {
                name: "Settings".to_string(),
                category: "Config".to_string(),
                count: 40,
            },
        ];

        // Test filtering by name
        let filtered = filter_items(&items, Some("event"), |item| &item.name);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&&items[1])); // UserEvents
        assert!(filtered.contains(&&items[2])); // EventLog

        // Test filtering by category
        let filtered = filter_items(&items, Some("events"), |item| &item.category);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&&items[1])); // UserEvents
        assert!(filtered.contains(&&items[2])); // EventLog

        // Test filtering with wildcard
        let filtered = filter_items(&items, Some("*Log"), |item| &item.name);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.contains(&&items[2])); // EventLog

        // Test filtering with no matches
        let filtered = filter_items(&items, Some("NonExistent"), |item| &item.name);
        assert_eq!(filtered.len(), 0);

        // Test with None pattern (should return all items)
        let filtered = filter_items(&items, None, |item| &item.name);
        assert_eq!(filtered.len(), items.len());
    }
}
