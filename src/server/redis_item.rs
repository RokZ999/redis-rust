use std::time::SystemTime;

/// Represents an item in a Redis-like database with optional expiration.
#[derive(Debug)]
pub struct RedisItem {
    data: String,
    expiration: Option<SystemTime>,
}

impl RedisItem {
    /// Creates a new `RedisItem` with the given data and no expiration.
    ///
    /// # Arguments
    ///
    /// * `data` - A `String` containing the data for the `RedisItem`.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `RedisItem` with no expiration set.
    ///
    /// # Examples
    ///
    /// ```
    /// let item = RedisItem::new("value".to_string());
    /// ```
    pub fn new(data: String) -> Self {
        RedisItem {
            data,
            expiration: None,
        }
    }

    /// Creates a new `RedisItem` with the given data and expiration time.
    ///
    /// # Arguments
    ///
    /// * `data` - A `String` containing the data for the `RedisItem`.
    /// * `expiration` - A `SystemTime` representing the expiration time for the `RedisItem`.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `RedisItem` with the specified expiration time.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::{SystemTime, Duration};
    ///
    /// let expiration = SystemTime::now() + Duration::from_secs(60);
    /// let item = RedisItem::new_with_expiration("value".to_string(), expiration);
    /// ```
    pub fn new_with_expiration(data: String, expiration: SystemTime) -> Self {
        RedisItem {
            data,
            expiration: Some(expiration),
        }
    }


    /// Checks whether the `RedisItem` has expired.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current time is later than the expiration time, or `false` if the item has not expired or has no expiration set.
    ///
    /// # Examples
    ///
    /// ```
    /// let item = RedisItem::new("value".to_string());
    /// assert!(!item.is_expired());
    /// ```
    pub fn is_expired(&self) -> bool {
        if let Some(expiration_time) = self.expiration {
            SystemTime::now() > expiration_time
        } else {
            false
        }
    }

    /// Retrieves the data stored in the `RedisItem`.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `String` containing the data.
    ///
    /// # Examples
    ///
    /// ```
    /// let item = RedisItem::new("value".to_string());
    /// assert_eq!(item.get_data(), "value");
    /// ```
    pub fn get_data(&self) -> &String {
        &self.data
    }
}
