#[macro_export]
macro_rules! impl_rkyv_for_pod {
    ($name:ident) => {
        #[cfg(feature = "rkyv")]
        const _: () = {
            use $crate::rkyv::{Archive, Archived, Deserialize, Fallible, Serialize};

            impl Archive for $name {
                type Archived = $name;
                type Resolver = ();

                #[inline]
                unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, out: *mut Self::Archived) {
                    *out = *self;
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
}

#[macro_export]
macro_rules! impl_serde_for_bitflags {
    ($name:ident) => {
        $crate::serde_shims::impl_serde_for_bitflags!($name);

        #[cfg(feature = "rkyv")]
        $crate::impl_rkyv_for_pod!($name);
    };
}

#[macro_export]
macro_rules! enum_codes {
    (
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

        #[cfg(feature = "rkyv")]
        const _: () = {
            use $crate::rkyv::{Archive, Deserialize, Fallible, Serialize, Archived};

            impl Archive for $name {
                type Archived = $repr;
                type Resolver = ();

                #[inline]
                unsafe fn resolve(&self, _pos: usize, _resolver: Self::Resolver, out: *mut Self::Archived) {
                    *out = *self as $repr;
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
                    Ok(match *self {
                        $($code => $name::$variant,)*
                        $(_     => $name::$unknown,)?

                        _ => panic!("Unknown code: {self}"),
                    })
                }
            }
        };
    };
}
