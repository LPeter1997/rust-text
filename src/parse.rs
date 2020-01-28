
// Binary parser utilities.

/// The result of parsing.
pub(crate) type ParseResult<T> = Result<T, ()>;

/// The trait that every parse-able type implements.
pub(crate) trait Parse: Sized {
    /// Parses the type from a little-endian input.
    fn parse_le(_input: &mut &[u8]) -> ParseResult<Self> { unimplemented!(); }
    /// Parses the type from a big-endian input.
    fn parse_be(_input: &mut &[u8]) -> ParseResult<Self> { unimplemented!(); }
}

// Macro to implement for integral types.

macro_rules! parseable_integral {
    ($t:ty) => {
        impl Parse for $t {
            fn parse_le(input: &mut &[u8]) -> ParseResult<Self> {
                let bytes = *input;
                // Bounds check
                const LEN: usize = std::mem::size_of::<$t>();
                if bytes.len() < LEN {
                    return Err(());
                }
                // Parse
                let mut result = <$t>::default();
                for i in 0..LEN {
                    result |= ((bytes[i] as $t) << (i * 8));
                }
                *input = &bytes[LEN..];
                Ok(Self::from_le(result))
            }

            fn parse_be(input: &mut &[u8]) -> ParseResult<Self> {
                let bytes = *input;
                // Bounds check
                const LEN: usize = std::mem::size_of::<$t>();
                if bytes.len() < LEN {
                    return Err(());
                }
                // Parse
                let mut result = <$t>::default();
                for i in 0..LEN {
                    result |= ((bytes[i] as $t) << (i * 8));
                }
                *input = &bytes[LEN..];
                Ok(Self::from_be(result))
            }
        }
    };
}

parseable_integral!(u8);
parseable_integral!(u16);
parseable_integral!(u32);
parseable_integral!(u64);

parseable_integral!(i8);
parseable_integral!(i16);
parseable_integral!(i32);
parseable_integral!(i64);

// Macro to implement parse for arrays.

macro_rules! parseable_array {
    ($n:expr) => {
        impl <T: Default + Parse> Parse for [T; $n] {
            fn parse_le(input: &mut &[u8]) -> ParseResult<Self> {
                let mut bytes = *input;
                let mut result: Self = Default::default();
                for i in 0..$n {
                    result[i] = T::parse_le(&mut bytes)?;
                }
                *input = bytes;
                Ok(result)
            }

            fn parse_be(input: &mut &[u8]) -> ParseResult<Self> {
                let mut bytes = *input;
                let mut result: Self = Default::default();
                for i in 0..$n {
                    result[i] = T::parse_be(&mut bytes)?;
                }
                *input = bytes;
                Ok(result)
            }
        }
    };
}

parseable_array!(0);
parseable_array!(1);
parseable_array!(2);
parseable_array!(3);
parseable_array!(4);

/// A macro that helps generating structures with Parse implementation.
#[macro_export]
macro_rules! parseable_struct {
    ($name:ident { $( $fname:ident : $fty:ty ),* $(,)? }) => {
        #[repr(C)]
        #[derive(Debug, Default, Clone)]
        struct $name {
            $( $fname : $fty ),*
        }
        impl Parse for $name {
            fn parse_le(input: &mut &[u8]) -> ParseResult<Self> {
                let mut bytes = *input;
                $( let $fname = <$fty>::parse_le(&mut bytes)?; )*
                *input = bytes;
                Ok(Self{
                    $( $fname ),*
                })
            }

            fn parse_be(input: &mut &[u8]) -> ParseResult<Self> {
                let mut bytes = *input;
                $( let $fname = <$fty>::parse_be(&mut bytes)?; )*
                *input = bytes;
                Ok(Self{
                    $( $fname ),*
                })
            }
        }
    };
}
