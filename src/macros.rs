#[macro_export]
macro_rules! convert_err_to {
    ($a:ident <- $($b: ident),+) => {
        pub enum $a {
            $(
                $b($b),
            )+
        }

        impl std::fmt::Debug for $a {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$b(b) => <$b as std::fmt::Debug>::fmt(b, f),
                    )+
                }
            }
        }

        impl std::fmt::Display for $a {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$b(b) => <$b as std::fmt::Display>::fmt(b, f),
                    )+
                }
            }
        }

        impl std::error::Error for $a {}

        $(
            impl std::convert::From<$b> for $a {
                fn from(b: $b) -> $a { $a::$b(b) }
            }
        )+
    };
}
