/// Defines a newtype ID wrapper around an integer-like scalar (typically `i64`)
/// and generates:
/// - `#[pyo3::pyclass(module = "...")]`
/// - derives (Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
/// - `Display`
/// - `From<$inner> for $name` and `From<$name> for $inner`
/// - `tiberius::{FromSql, IntoSql, ToSql}` using the inner type’s implementations
///
/// Usage:
///   define_id_type!(i64, ScheduleId);
///   // optionally override the pyo3 module:
///   define_id_type!(i64, ScheduleId, module = "tsi_rust_api");
#[macro_export]
macro_rules! define_id_type {
    ($inner:ty, $name:ident) => {
        $crate::define_id_type!($inner, $name, module = "tsi_rust_api");
    };

    ($inner:ty, $name:ident, module = $module:literal) => {
        #[pyo3::pyclass(module = $module)]
        #[derive(
            Debug,
            Copy,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(pub $inner);

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::write!(f, "{}", self.0)
            }
        }

        impl ::std::convert::From<$inner> for $name {
            fn from(v: $inner) -> Self {
                $name(v)
            }
        }

        impl ::std::convert::From<$name> for $inner {
            fn from(v: $name) -> Self {
                v.0
            }
        }

        #[pyo3::pymethods]
        impl $name {
            #[new]
            pub fn new(value: $inner) -> Self {
                $name(value)
            }

            #[getter]
            pub fn value(&self) -> $inner {
                self.0
            }
        }
    };
}

// Example:
// define_id_type!(i64, ScheduleId);
// define_id_type!(i64, UserId, module = "tsi_rust_api");
