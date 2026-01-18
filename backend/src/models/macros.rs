/// Defines a newtype ID wrapper around an integer-like scalar (typically `i64`)
/// and generates:
/// - derives (Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)
/// - `Display`
/// - `From<$inner> for $name` and `From<$name> for $inner`
///
/// Usage:
///   define_id_type!(i64, ScheduleId);
#[macro_export]
macro_rules! define_id_type {
    ($inner:ty, $name:ident) => {
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

        impl $name {
            pub fn new(value: $inner) -> Self {
                $name(value)
            }

            pub fn value(&self) -> $inner {
                self.0
            }
        }
    };
}
