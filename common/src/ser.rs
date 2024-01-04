#[cfg(feature = "rkyv")]
#[macro_export]
macro_rules! impl_rkyv_for_pod {
    ($name:ident) => {
        const _: () = {
            use $crate::rkyv::{Archive, Archived, Deserialize, Fallible, Serialize};

            impl Archive for $name {
                type Archived = $name;
                type Resolver = ();

                #[inline]
                unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, out: *mut Self::Archived) {
                    *out = *self; // asserts copy
                }
            }

            impl<S: Fallible + ?Sized> Serialize<S> for $name {
                #[inline]
                fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                    Ok(())
                }
            }

            impl<D: Fallible + ?Sized> Deserialize<$name, D> for Archived<$name> {
                fn deserialize(&self, _deserializer: &mut D) -> Result<$name, D::Error> {
                    Ok(*self)
                }
            }
        };
    };

    ($name:ident +CheckBytes) => {
        $crate::impl_rkyv_for_pod!($name);

        const _: () = {
            use rkyv::bytecheck::CheckBytes;

            impl<C: ?Sized> CheckBytes<C> for $name {
                type Error = ::core::convert::Infallible;

                #[inline(always)]
                unsafe fn check_bytes<'a>(value: *const Self, context: &mut C) -> Result<&'a Self, Self::Error> {
                    Ok(&*value)
                }
            }
        };
    };
}

#[cfg(not(feature = "rkyv"))]
#[macro_export]
macro_rules! impl_rkyv_for_pod {
    ($($tt:tt)*) => {};
}

#[cfg(feature = "rkyv")]
#[macro_export]
macro_rules! impl_serde_for_bitflags {
    ($name:ident) => {
        $crate::serde_shims::impl_serde_for_bitflags!($name);

        $crate::impl_rkyv_for_pod!($name + CheckBytes);
    };
}

#[cfg(not(feature = "rkyv"))]
#[macro_export]
macro_rules! impl_serde_for_bitflags {
    ($name:ident) => {
        $crate::serde_shims::impl_serde_for_bitflags!($name);
    };
}

#[doc(hidden)]
#[cfg(not(feature = "rkyv"))]
#[macro_export]
macro_rules! impl_rkyv_for_enum_codes {
    ($($tt:tt)*) => {};
}

#[doc(hidden)]
#[cfg(feature = "rkyv")]
#[macro_export]
macro_rules! impl_rkyv_for_enum_codes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ty $(= $unknown:ident)? {$(
            $(#[$variant_meta:meta])*
            $variant:ident = $code:expr,
        )*}
    ) => {
        const _: () = {
            use $crate::rkyv::{Archive, Deserialize, Fallible, Serialize, Archived};

            impl Archive for $name {
                type Archived = $crate::rend::LittleEndian<$repr>;
                type Resolver = ();

                #[inline]
                unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, out: *mut Self::Archived) {
                    *out = $crate::rend::LittleEndian::<$repr>::new(*self as $repr);
                }
            }

            impl<S: Fallible + ?Sized> Serialize<S> for $name {
                #[inline]
                fn serialize(&self, _serializer: &mut S) -> Result<Self::Resolver, S::Error> {
                    Ok(())
                }
            }

            impl<D: Fallible + ?Sized> Deserialize<$name, D> for Archived<$name> {
                fn deserialize(&self, _deserializer: &mut D) -> Result<$name, D::Error> {
                    Ok(match self.value() {
                        $($code => $name::$variant,)*
                        $(_     => $name::$unknown,)?

                        u @ _ => panic!("Unknown code: {u}"),
                    })
                }
            }
        };
    }
}

#[macro_export]
macro_rules! enum_codes {
    (
        @ENTRY
        $(#[$meta:meta])*
        $vis:vis enum $name:ident: $repr:ty $(= $unknown:ident)? {$(
            $(#[$variant_meta:meta])*
            $variant:ident = $code:expr,
        )*}
    ) => {
        $(#[$meta])*
        #[repr($repr)]
        $vis enum $name {$(
            $(#[$variant_meta])*
            $variant = $code,
        )*}
    };

    ($($tt:tt)*) => {
        $crate::enum_codes!(@ENTRY $($tt)*);

        $crate::impl_rkyv_for_enum_codes!($($tt)*);
    };
}
