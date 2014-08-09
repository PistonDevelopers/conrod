
#![macro_escape]

/// Simplify implementation of ToPrimitive.
#[macro_export]
macro_rules! widget_state(
    ($obj:ty, $obji:ident {
        $($var:ident -> $val:expr),+
    }) => (

        /// Widget state.
        #[deriving(FromPrimitive)]
        pub enum $obji {
            $($var, )+
        }

        impl ToPrimitive for $obj {
            fn to_i64(&self) -> Option<i64> {
                match self {
                    $(&$var => Some($val as i64),)+
                    //_ => None,
                }
            }
            fn to_u64(&self) -> Option<u64> {
                match self {
                    $(&$var => Some($val as u64),)+
                    //_ => None,
                }
            }
        }
    )
)

