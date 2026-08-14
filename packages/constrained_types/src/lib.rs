// SPDX-License-Identifier: Apache-2.0

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

//! Generic constrained types for validated data
//!
//! This module provides generic types for string and byte data with compile-time
//! length constraints, useful for validating protocol data across multiple chains.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Deref;

/// Errors for constrained type validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstrainedError {
    TooShort,
    TooLong,
    Empty,
}

impl core::fmt::Display for ConstrainedError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooShort => write!(f, "Value too short"),
            Self::TooLong => write!(f, "Value too long"),
            Self::Empty => write!(f, "Value is empty"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConstrainedError {}

/// Generic constrained string type
///
/// A string wrapper that enforces minimum and maximum length constraints at compile time.
///
/// # Type Parameters
/// * `MIN` - Minimum length in bytes (inclusive)
/// * `MAX` - Maximum length in bytes (inclusive)
///
/// # Examples
/// ```
/// use ibc_eureka_constrained_types::{ConstrainedString, ConstrainedError};
///
/// type Username = ConstrainedString<3, 20>;
///
/// let valid = Username::new("alice").unwrap();
/// assert_eq!(&*valid, "alice");
///
/// let too_short = Username::new("ab");
/// assert!(too_short.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize))]
pub struct ConstrainedString<const MIN: usize, const MAX: usize>(String);

impl<const MIN: usize, const MAX: usize> ConstrainedString<MIN, MAX> {
    /// Create a new constrained string with validation
    pub fn new(s: impl Into<String>) -> Result<Self, ConstrainedError> {
        let s = s.into();
        match s.len() {
            0 if MIN > 0 => Err(ConstrainedError::Empty),
            len if len < MIN => Err(ConstrainedError::TooShort),
            len if len > MAX => Err(ConstrainedError::TooLong),
            _ => Ok(Self(s)),
        }
    }

    /// Consume and return the inner `String`
    pub fn into_string(self) -> String {
        self.0
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<str> for ConstrainedString<MIN, MAX> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const MIN: usize, const MAX: usize> Deref for ConstrainedString<MIN, MAX> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MIN: usize, const MAX: usize> core::fmt::Display for ConstrainedString<MIN, MAX> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const MIN: usize, const MAX: usize> core::convert::TryFrom<String>
    for ConstrainedString<MIN, MAX>
{
    type Error = ConstrainedError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<const MIN: usize, const MAX: usize> core::convert::TryFrom<&str>
    for ConstrainedString<MIN, MAX>
{
    type Error = ConstrainedError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(String::from(s))
    }
}

/// Generic constrained vector type
///
/// A vector wrapper that enforces minimum and maximum length constraints at compile time.
/// Unlike `ConstrainedBytes`, this works with any type `T`.
///
/// # Type Parameters
/// * `T` - The element type
/// * `MIN` - Minimum length (inclusive)
/// * `MAX` - Maximum length (inclusive)
///
/// # Examples
/// ```
/// use ibc_eureka_constrained_types::{ConstrainedVec, ConstrainedError};
///
/// type Payload = ConstrainedVec<u8, 1, 1024>;
///
/// let valid = Payload::new(vec![1, 2, 3]).unwrap();
/// assert_eq!(&*valid, &[1, 2, 3]);
///
/// let empty = Payload::new(vec![]);
/// assert!(empty.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstrainedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> ConstrainedVec<T, MIN, MAX> {
    /// Create new constrained vector with validation
    pub fn new(vec: impl Into<Vec<T>>) -> Result<Self, ConstrainedError> {
        let vec = vec.into();
        match vec.len() {
            0 if MIN > 0 => Err(ConstrainedError::Empty),
            len if len < MIN => Err(ConstrainedError::TooShort),
            len if len > MAX => Err(ConstrainedError::TooLong),
            _ => Ok(Self(vec)),
        }
    }

    /// Consume and return the inner `Vec<T>`
    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T, const MAX: usize> ConstrainedVec<T, 0, MAX> {
    #[must_use]
    pub fn empty() -> Self {
        Self(Vec::new())
    }
}

impl<T, const MIN: usize, const MAX: usize> AsRef<[T]> for ConstrainedVec<T, MIN, MAX> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> Deref for ConstrainedVec<T, MIN, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const MIN: usize, const MAX: usize> core::convert::TryFrom<Vec<T>>
    for ConstrainedVec<T, MIN, MAX>
{
    type Error = ConstrainedError;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(vec)
    }
}

/// Generic constrained bytes type
///
/// A byte vector wrapper that enforces minimum and maximum length constraints at compile time.
/// Internally uses `ConstrainedVec<u8, MIN, MAX>`.
///
/// # Type Parameters
/// * `MIN` - Minimum length in bytes (inclusive)
/// * `MAX` - Maximum length in bytes (inclusive)
///
/// # Examples
/// ```
/// use ibc_eureka_constrained_types::{ConstrainedBytes, ConstrainedError};
///
/// type Salt = ConstrainedBytes<0, 32>;
///
/// let valid = Salt::new(vec![1, 2, 3, 4]).unwrap();
/// assert_eq!(&*valid, &[1, 2, 3, 4]);
///
/// let too_long = Salt::new(vec![0u8; 33]);
/// assert!(too_long.is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstrainedBytes<const MIN: usize, const MAX: usize>(ConstrainedVec<u8, MIN, MAX>);

#[cfg(feature = "borsh")]
impl<const MIN: usize, const MAX: usize> borsh::BorshSerialize for ConstrainedBytes<MIN, MAX> {
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> borsh::maybestd::io::Result<()> {
        borsh::BorshSerialize::serialize(self.0.as_ref(), writer)
    }
}

impl<const MIN: usize, const MAX: usize> ConstrainedBytes<MIN, MAX> {
    /// Create new constrained bytes with validation
    pub fn new(bytes: impl Into<Vec<u8>>) -> Result<Self, ConstrainedError> {
        ConstrainedVec::new(bytes).map(Self)
    }

    /// Consume and return the inner `Vec<u8>`
    pub fn into_vec(self) -> Vec<u8> {
        self.0.into_vec()
    }
}

impl<const MAX: usize> ConstrainedBytes<0, MAX> {
    #[must_use]
    pub fn empty() -> Self {
        Self(ConstrainedVec::empty())
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<[u8]> for ConstrainedBytes<MIN, MAX> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl<const MIN: usize, const MAX: usize> Deref for ConstrainedBytes<MIN, MAX> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MIN: usize, const MAX: usize> core::convert::TryFrom<Vec<u8>>
    for ConstrainedBytes<MIN, MAX>
{
    type Error = ConstrainedError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new(bytes)
    }
}

impl<const MIN: usize, const MAX: usize> core::convert::TryFrom<&[u8]>
    for ConstrainedBytes<MIN, MAX>
{
    type Error = ConstrainedError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::new(Vec::from(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, vec};

    #[test]
    fn test_constrained_string_valid() {
        type Username = ConstrainedString<3, 20>;

        let username = Username::new("alice").unwrap();
        assert_eq!(&*username, "alice");
        assert_eq!(username.len(), 5);
    }

    #[test]
    fn test_constrained_string_too_short() {
        type Username = ConstrainedString<3, 20>;

        let result = Username::new("ab");
        assert_eq!(result, Err(ConstrainedError::TooShort));
    }

    #[test]
    fn test_constrained_string_too_long() {
        type Username = ConstrainedString<3, 20>;

        let result = Username::new("a".repeat(21));
        assert_eq!(result, Err(ConstrainedError::TooLong));
    }

    #[test]
    fn test_constrained_string_empty() {
        type Username = ConstrainedString<1, 20>;

        let result = Username::new("");
        assert_eq!(result, Err(ConstrainedError::Empty));
    }

    #[test]
    fn test_constrained_string_zero_min() {
        type MaybeEmpty = ConstrainedString<0, 20>;

        let empty = MaybeEmpty::new("").unwrap();
        assert_eq!(&*empty, "");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_constrained_vec_valid() {
        type Payload = ConstrainedVec<u8, 1, 10>;

        let payload = Payload::new(vec![1, 2, 3]).unwrap();
        assert_eq!(&*payload, &[1, 2, 3]);
        assert_eq!(payload.len(), 3);
    }

    #[test]
    fn test_constrained_vec_too_short() {
        type Payload = ConstrainedVec<u8, 3, 10>;

        let result = Payload::new(vec![1, 2]);
        assert_eq!(result, Err(ConstrainedError::TooShort));
    }

    #[test]
    fn test_constrained_vec_too_long() {
        type Payload = ConstrainedVec<u8, 1, 5>;

        let result = Payload::new(vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(result, Err(ConstrainedError::TooLong));
    }

    #[test]
    fn test_constrained_vec_empty_not_allowed() {
        type Payload = ConstrainedVec<u8, 1, 10>;

        let result = Payload::new(vec![]);
        assert_eq!(result, Err(ConstrainedError::Empty));
    }

    #[test]
    fn test_constrained_vec_empty_allowed() {
        type MaybeEmpty = ConstrainedVec<u8, 0, 10>;

        let empty = MaybeEmpty::new(vec![]).unwrap();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_constrained_vec_empty_method() {
        type MaybeEmpty = ConstrainedVec<u8, 0, 10>;

        let empty = MaybeEmpty::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_constrained_bytes_empty_method() {
        type Salt = ConstrainedBytes<0, 32>;

        let empty = Salt::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_constrained_vec_non_u8() {
        type Numbers = ConstrainedVec<i32, 2, 5>;

        let numbers = Numbers::new(vec![10, 20, 30]).unwrap();
        assert_eq!(&*numbers, &[10, 20, 30]);
        assert_eq!(numbers.len(), 3);
    }

    #[test]
    fn test_constrained_vec_into_vec() {
        type Payload = ConstrainedVec<u8, 1, 10>;

        let payload = Payload::new(vec![1, 2, 3]).unwrap();
        let inner = payload.into_vec();
        assert_eq!(inner, vec![1, 2, 3]);
    }

    #[test]
    fn test_constrained_bytes_valid() {
        type Salt = ConstrainedBytes<0, 32>;

        let salt = Salt::new(vec![1, 2, 3, 4]).unwrap();
        assert_eq!(&*salt, &[1, 2, 3, 4]);
        assert_eq!(salt.len(), 4);
    }

    #[test]
    fn test_constrained_bytes_too_long() {
        type Salt = ConstrainedBytes<0, 32>;

        let result = Salt::new(vec![0u8; 33]);
        assert_eq!(result, Err(ConstrainedError::TooLong));
    }

    #[test]
    fn test_constrained_bytes_display() {
        type Name = ConstrainedString<1, 10>;

        let name = Name::new("Alice").unwrap();
        assert_eq!(format!("{}", name), "Alice");
    }
}

/// Generic non-empty wrapper type
///
/// Validates that the wrapped data is not empty. Works with any type that has
/// a length/is_empty check.
///
/// # Examples
/// ```
/// use ibc_eureka_constrained_types::{NonEmpty, ConstrainedError};
///
/// let data = NonEmpty::<Vec<u8>>::new(vec![1, 2, 3]).unwrap();
/// assert_eq!(data.len(), 3);
///
/// let empty = NonEmpty::<Vec<u8>>::new(vec![]);
/// assert_eq!(empty, Err(ConstrainedError::Empty));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmpty<T>(T);

impl<T> NonEmpty<T> {
    /// Unwrap the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for NonEmpty<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> core::fmt::Display for NonEmpty<T>
where
    T: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Implementation for Vec<u8>
impl NonEmpty<Vec<u8>> {
    pub fn new(data: Vec<u8>) -> Result<Self, ConstrainedError> {
        if data.is_empty() {
            return Err(ConstrainedError::Empty);
        }
        Ok(Self(data))
    }
}

impl TryFrom<Vec<u8>> for NonEmpty<Vec<u8>> {
    type Error = ConstrainedError;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        Self::new(data)
    }
}

// Implementation for String
impl NonEmpty<String> {
    pub fn new(data: String) -> Result<Self, ConstrainedError> {
        if data.is_empty() {
            return Err(ConstrainedError::Empty);
        }
        Ok(Self(data))
    }
}

impl TryFrom<String> for NonEmpty<String> {
    type Error = ConstrainedError;

    fn try_from(data: String) -> Result<Self, Self::Error> {
        Self::new(data)
    }
}

#[cfg(test)]
mod non_empty_tests {
    use super::*;

    #[test]
    fn test_non_empty_vec_success() {
        let data = NonEmpty::<Vec<u8>>::new(vec![1, 2, 3]).unwrap();
        assert_eq!(&*data, &[1, 2, 3]);
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn test_non_empty_vec_empty() {
        let result = NonEmpty::<Vec<u8>>::new(vec![]);
        assert_eq!(result, Err(ConstrainedError::Empty));
    }

    #[test]
    fn test_non_empty_string_success() {
        let data = NonEmpty::<String>::new("hello".to_string()).unwrap();
        assert_eq!(&**data, "hello");
    }

    #[test]
    fn test_non_empty_string_empty() {
        let result = NonEmpty::<String>::new(String::new());
        assert_eq!(result, Err(ConstrainedError::Empty));
    }

    #[test]
    fn test_non_empty_display() {
        let data = NonEmpty::<String>::new("test".to_string()).unwrap();
        assert_eq!(format!("{}", data), "test");
    }

    #[test]
    fn test_non_empty_into_inner() {
        let data = NonEmpty::<Vec<u8>>::new(vec![1, 2, 3]).unwrap();
        let inner = data.into_inner();
        assert_eq!(inner, vec![1, 2, 3]);
    }
}
