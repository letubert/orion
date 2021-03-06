// MIT License

// Copyright (c) 2018-2019 The orion Developers

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[cfg(feature = "safe_api")]
/// Macro that implements the `Default` trait, which will make a type, that
/// needs secure default methods like CSPRNG generation, return itself with a
/// default and secure length of random bytes.
macro_rules! impl_default_trait (($name:ident, $size:expr) => (
    impl core::default::Default for $name {
        #[must_use]
        #[cfg(feature = "safe_api")]
        /// Randomly generate using a CSPRNG with recommended size. Not available in `no_std` context.
        fn default() -> $name {
            use crate::util;
            let mut value = vec![0u8; $size];
            util::secure_rand_bytes(&mut value).unwrap();

            $name { value: value }
        }
    }
));

/// Macro that implements the `PartialEq` trait on a object called `$name` that
/// also implements `unprotected_as_bytes()`. This `PartialEq` will perform in
/// constant time.
macro_rules! impl_ct_partialeq_trait (($name:ident) => (
    impl PartialEq for $name {
        fn eq(&self, other: &$name) -> bool {
            use subtle::ConstantTimeEq;
             self.unprotected_as_bytes()
                .ct_eq(&other.unprotected_as_bytes())
                .unwrap_u8() == 1
        }
    }
));

/// Macro that implements the `PartialEq` trait on a object called `$name` that
/// also implements `as_bytes()`. This `PartialEq` will NOT perform in
/// constant time.
macro_rules! impl_normal_partialeq_trait (($name:ident) => (
    impl PartialEq for $name {
        fn eq(&self, other: &$name) -> bool {
            (&self.as_bytes() == &other.as_bytes())
        }
    }
));

/// Macro that implements the `Debug` trait on a object called `$name`.
/// This `Debug` will omit any fields of object `$name` to avoid them being
/// written to logs.
macro_rules! impl_omitted_debug_trait (($name:ident) => (
    impl core::fmt::Debug for $name {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "{} {{***OMITTED***}}", stringify!($name))
        }
    }
));

/// Macro that implements the `Debug` trait on a object called `$name`.
/// This `Debug` will omit any fields of object `$name` to avoid them being
/// written to logs.
macro_rules! impl_normal_debug_trait (($name:ident) => (
    impl core::fmt::Debug for $name {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "{} {:?}", stringify!($name), &self.value[..])
        }
    }
));

/// Macro that implements the `Drop` trait on a object called `$name` which as a
/// field `value`. This `Drop` will zero out the field `value` when the objects
/// destructor is called. WARNING: This requires value to be an array as
/// clear_on_drop will not be called correctly if this particluar trait is
/// implemented on Vec's.
macro_rules! impl_drop_trait (($name:ident) => (
    impl Drop for $name {
        fn drop(&mut self) {
            use zeroize::Zeroize;
            self.value.zeroize();
        }
    }
));

/// Macro to implement a `from_slice()` function. Returns `UnknownCryptoError`
/// if the slice is not of length `$size`.
macro_rules! func_from_slice (($name:ident, $size:expr) => (
    #[must_use]
    /// Make an object from a given byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<$name, UnknownCryptoError> {
        if slice.len() != $size {
            return Err(UnknownCryptoError);
        }

        let mut value = [0u8; $size];
        value.copy_from_slice(slice);

        Ok($name { value: value })
    }
));

#[cfg(feature = "safe_api")]
/// Macro to implement a `from_slice()` function. Returns `UnknownCryptoError`
/// if the slice is not of length `$size`.
macro_rules! func_from_slice_variable_size (($name:ident) => (
    #[must_use]
    #[cfg(feature = "safe_api")]
    /// Make an object from a given byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<$name, UnknownCryptoError> {
        if slice.is_empty() {
            return Err(UnknownCryptoError);
        }

        Ok($name { value: Vec::from(slice) })
    }
));

/// Macro to implement a `unprotected_as_bytes()` function for objects that
/// implement extra protections. Typically used on objects that implement
/// `Drop`, `Debug` and/or `PartialEq`.
macro_rules! func_unprotected_as_bytes (() => (
    #[must_use]
    /// Return the object as byte slice. __**Warning**__: Should not be used unless strictly
    /// needed. This __**breaks protections**__ that the type implements.
    pub fn unprotected_as_bytes(&self) -> &[u8] {
        self.value.as_ref()
    }
));

/// Macro to implement a `as_bytes()` function for objects that don't implement
/// extra protections.
macro_rules! func_as_bytes (() => (
    #[must_use]
    /// Return the object as byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        self.value.as_ref()
    }
));

/// Macro to implement a `get_length()` function which will return the objects'
/// length of field `value`.
macro_rules! func_get_length (() => (
    /// Return the length of the object.
    pub fn get_length(&self) -> usize {
        self.value.len()
    }
));

/// Macro to implement a `generate()` function for objects that benefit from
/// having a CSPRNG available to generate data of a fixed length $size.
macro_rules! func_generate (($name:ident, $size:expr) => (
    #[must_use]
    #[cfg(feature = "safe_api")]
    /// Randomly generate using a CSPRNG. Not available in `no_std` context.
    pub fn generate() -> Result<$name, UnknownCryptoError> {
        use crate::util;
        let mut value = [0u8; $size];
        util::secure_rand_bytes(&mut value)?;

        Ok($name { value: value })
    }
));

#[cfg(feature = "safe_api")]
/// Macro to implement a `generate()` function for objects that benefit from
/// having a CSPRNG available to generate data of a variable length.
macro_rules! func_generate_variable_size (($name:ident) => (
    #[must_use]
    #[cfg(feature = "safe_api")]
    /// Randomly generate using a CSPRNG. Not available in `no_std` context.
    pub fn generate(length: usize) -> Result<$name, UnknownCryptoError> {
        use crate::util;
        if length < 1 || length >= (u32::max_value() as usize) {
            return Err(UnknownCryptoError);
        }

        let mut value = vec![0u8; length];
        util::secure_rand_bytes(&mut value)?;

        Ok($name { value: value })
    }
));

/// Macro to construct a type containing sensitive data, using a fixed-size
/// array.
macro_rules! construct_secret_key {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name { value: [u8; $size] }

        impl_omitted_debug_trait!($name);
        impl_drop_trait!($name);
        impl_ct_partialeq_trait!($name);

        impl $name {
            func_from_slice!($name, $size);
            func_unprotected_as_bytes!();
            func_generate!($name, $size);
            func_get_length!();
        }

        #[test]
        fn test_key_size() {
            assert!($name::from_slice(&[0u8; $size]).is_ok());
            assert!($name::from_slice(&[0u8; 0]).is_err());
            assert!($name::from_slice(&[0u8; $size - 1]).is_err());
            assert!($name::from_slice(&[0u8; $size + 1]).is_err());
        }
        #[test]
        fn test_unprotected_as_bytes_secret_key() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes() == &[0u8; $size]);
            assert!(test.unprotected_as_bytes().len() == $size);
        }

        #[test]
        fn test_get_length_secret_key() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_secret_key() {
            let test_zero = $name::from_slice(&[0u8; $size]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate().unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == $size);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_secret_key() {
            let secret = format!("{:?}", [0u8; $size].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; $size]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}

/// Macro to construct a nonce where a random generator is not applicable.
macro_rules! construct_nonce_no_generator {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        $(#[$meta])*
        pub struct $name { value: [u8; $size] }

        impl_normal_debug_trait!($name);
        impl_normal_partialeq_trait!($name);

        impl $name {
            func_from_slice!($name, $size);
            func_as_bytes!();
            func_get_length!();
        }

        #[test]
        fn test_nonce_size() {
            assert!($name::from_slice(&[0u8; $size]).is_ok());
            assert!($name::from_slice(&[0u8; $size - $size]).is_err());
            assert!($name::from_slice(&[0u8; $size + 1]).is_err());
        }
        #[test]
        fn test_as_bytes_nonce_no_gen() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.as_bytes().len() == $size);
            assert!(test.as_bytes() == [0u8; $size].as_ref());
        }

        #[test]
        fn test_get_length_nonce_no_gen() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }
    );
}

/// Macro to construct a nonce where a random generator is applicable.
macro_rules! construct_nonce_with_generator {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        $(#[$meta])*
        pub struct $name { value: [u8; $size] }

        impl_normal_debug_trait!($name);
        impl_normal_partialeq_trait!($name);

        impl $name {
            func_from_slice!($name, $size);
            func_as_bytes!();
            func_generate!($name, $size);
            func_get_length!();
        }

        #[test]
        fn test_nonce_size() {
            assert!($name::from_slice(&[0u8; $size]).is_ok());
            assert!($name::from_slice(&[0u8; $size - $size]).is_err());
            assert!($name::from_slice(&[0u8; $size + 1]).is_err());
        }
        #[test]
        fn test_as_bytes_nonce_with_gen() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.as_bytes().len() == $size);
            assert!(test.as_bytes() == &[0u8; $size]);
        }

        #[test]
        fn test_get_length_nonce_with_gen() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_nonce() {
            let test_zero = $name::from_slice(&[0u8; $size]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate().unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == $size);
        }
    );
}

/// Macro to construct a tag type that MACs return.
macro_rules! construct_tag {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name { value: [u8; $size] }

        impl_omitted_debug_trait!($name);
        impl_ct_partialeq_trait!($name);

        impl $name {
            func_from_slice!($name, $size);
            func_unprotected_as_bytes!();
            func_get_length!();
        }

        #[test]
        fn test_tag_size() {
            assert!($name::from_slice(&[0u8; $size]).is_ok());
            assert!($name::from_slice(&[0u8; $size - $size]).is_err());
            assert!($name::from_slice(&[0u8; $size + 1]).is_err());
        }
        #[test]
        fn test_unprotected_as_bytes_tag() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes().len() == $size);
            assert!(test.unprotected_as_bytes() == [0u8; $size].as_ref());
        }

        #[test]
        fn test_get_length_tag() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_tag() {
            let secret = format!("{:?}", [0u8; $size].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; $size]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}

/// Macro to construct a secret key used for HMAC. This pre-pads the given key
/// to the required length specified by the HMAC specifications.
macro_rules! construct_hmac_key {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name { value: [u8; $size] }

        impl_omitted_debug_trait!($name);
        impl_drop_trait!($name);
        impl_ct_partialeq_trait!($name);

        impl $name {
            #[must_use]
            /// Make an object from a given byte slice.
            pub fn from_slice(slice: &[u8]) -> Result<$name, UnknownCryptoError> {
                use crate::hazardous::hash::sha512;
                use crate::hazardous::constants::SHA512_OUTSIZE;

                let mut secret_key = [0u8; $size];

                let slice_len = slice.len();

                if slice_len > $size {
                    secret_key[..SHA512_OUTSIZE].copy_from_slice(&sha512::digest(slice)?.as_bytes());
                } else {
                    secret_key[..slice_len].copy_from_slice(slice);
                }

                Ok($name { value: secret_key })
            }

            func_unprotected_as_bytes!();
            func_generate!($name, $size);
            func_get_length!();
        }

        #[test]
        fn test_key_size() {
            assert!($name::from_slice(&[0u8; $size]).is_ok());
            assert!($name::from_slice(&[0u8; $size - $size]).is_ok());
            assert!($name::from_slice(&[0u8; $size + 1]).is_ok());
        }

        #[test]
        fn test_unprotected_as_bytes_hmac_key() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes().len() == $size);
            assert!(test.unprotected_as_bytes() == [0u8; $size].as_ref());
        }

        #[test]
        fn test_get_length_hmac_key() {
            let test = $name::from_slice(&[0u8; $size]).unwrap();
            assert!(test.unprotected_as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_hmac() {
            let test_zero = $name::from_slice(&[0u8; $size]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate().unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == $size);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_hmac_key() {
            let secret = format!("{:?}", [0u8; $size].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; $size]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}

/// Macro to construct a secret key used for BLAKE2b. It is padded aginst a
/// BLOCKSIZE value, but can at most be half that when generated or constructed
/// from a slice.
macro_rules! construct_blake2b_key {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name {
            value: [u8; $size],
            original_size: usize,
        }

        impl_omitted_debug_trait!($name);
        impl_drop_trait!($name);
        impl_ct_partialeq_trait!($name);

        impl $name {
            #[must_use]
            /// Make an object from a given byte slice.
            pub fn from_slice(slice: &[u8]) -> Result<$name, UnknownCryptoError> {
                if slice.len() > 64 || slice.is_empty() {
                    return Err(UnknownCryptoError);
                }

                let mut secret_key = [0u8; $size];
                let slice_len = slice.len();
                secret_key[..slice_len].copy_from_slice(slice);

                Ok($name {
                    value: secret_key,
                    original_size: slice_len,
                })
            }

            /// Get the original size of the key, before padding.
            pub fn get_original_length(&self) -> usize {
                self.original_size
            }

            #[must_use]
            #[cfg(feature = "safe_api")]
            /// Randomly generate using a CSPRNG. Not available in `no_std` context.
            pub fn generate() -> Result<$name, UnknownCryptoError> {
                use crate::util;
                let mut value = [0u8; $size];
                // BLAKE2b key can be at max 64 bytes
                util::secure_rand_bytes(&mut value[..64])?;

                Ok($name {
                    value: value,
                    original_size: 64,
                })
            }

            func_unprotected_as_bytes!();
            func_get_length!();
        }

        #[test]
        fn test_blake2b_key_size() {
            assert!($name::from_slice(&[0u8; 64]).is_ok());
            assert!($name::from_slice(&[0u8; 63]).is_ok());
            assert!($name::from_slice(&[0u8; 65]).is_err());
            assert!($name::from_slice(&[0u8; 0]).is_err());
        }
        #[test]
        fn test_unprotected_as_bytes_blake2b_key() {
            // Verify against $size because that's how the key is padded.
            let test = $name::from_slice(&[0u8; 64]).unwrap();
            assert!(test.unprotected_as_bytes() == [0u8; $size].as_ref());

            let test = $name::from_slice(&[0u8; 32]).unwrap();
            assert!(test.unprotected_as_bytes() == [0u8; $size].as_ref());

            let test = $name::from_slice(&[0u8; 1]).unwrap();
            assert!(test.unprotected_as_bytes() == [0u8; $size].as_ref());
        }

        #[test]
        fn test_get_length_blake2b_key() {
            let test = $name::from_slice(&[0u8; 64]).unwrap();
            assert!(test.unprotected_as_bytes().len() == test.get_length());
            assert!($size == test.get_length());
        }

        #[test]
        fn test_get_original_length_blake2b_key() {
            let test = $name::from_slice(&[0u8; 64]).unwrap();
            assert!(test.unprotected_as_bytes().len() != test.get_original_length());
            assert!(64 == test.get_original_length());
        }

        #[test]
        fn test_blake2b_key_zero_padding() {
            let test = $name::from_slice(&[1u8; 64]).unwrap();
            assert!([0u8; 64].as_ref() == &test.unprotected_as_bytes()[64..]);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_blake2b() {
            let test_zero = $name::from_slice(&[0u8; 64]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate().unwrap();
            assert!(test_zero != test_rand);
            assert_ne!(
                test_zero.unprotected_as_bytes(),
                &test_rand.unprotected_as_bytes()[..test_zero.get_original_length()]
            );
            // A random generated one should always be 64 in length for Blake2b.
            assert!(test_rand.get_original_length() == 64);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_blake2b_key() {
            let secret = format!("{:?}", [0u8; 64].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; 64]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}

/// Macro to construct a digest returned by BLAKE2b.
macro_rules! construct_digest {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        #[derive(Clone, Copy)]
        $(#[$meta])*
        ///
        pub struct $name {
            value: [u8; $size],
            digest_size: usize,
        }

        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                use subtle::ConstantTimeEq;
                 self.as_bytes()
                    .ct_eq(&other.as_bytes())
                    .unwrap_u8() == 1
            }
        }

        impl $name {
            #[must_use]
            /// Return the object as byte slice.
            pub fn as_bytes(&self) -> &[u8] {
                self.value[..self.digest_size].as_ref()
            }

            #[must_use]
            /// Make an object from a given byte slice.
            pub fn from_slice(slice: &[u8]) -> Result<$name, UnknownCryptoError> {
                if slice.is_empty() || slice.len() > $size {
                    return Err(UnknownCryptoError);
                }

                let mut value = [0u8; $size];
                value[..slice.len()].copy_from_slice(slice);

                Ok($name {
                    value: value,
                    digest_size: slice.len(),
                })
            }

            /// Return the length of the object.
            pub fn get_length(&self) -> usize {
                self.digest_size
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "{} value: {:?}, digest_size: {:?}", stringify!($name), &self.value[..], &self.digest_size)
            }
        }

        #[test]
        fn test_blake2b_digest_size() {
            assert!($name::from_slice(&[0u8; 64]).is_ok());
            assert!($name::from_slice(&[0u8; 63]).is_ok());
            assert!($name::from_slice(&[0u8; 65]).is_err());
            assert!($name::from_slice(&[0u8; 0]).is_err());
        }
        #[test]
        fn test_as_bytes_blake2b_digest() {
            let test = $name::from_slice(&[0u8; 64]).unwrap();
            assert!(test.as_bytes() == [0u8; 64].as_ref());

            let test = $name::from_slice(&[0u8; 32]).unwrap();
            assert!(test.as_bytes() == [0u8; 32].as_ref());

            let test = $name::from_slice(&[0u8; 1]).unwrap();
            assert!(test.as_bytes() == [0u8; 1].as_ref());
        }

        #[test]
        fn test_get_length_blake2b_digest() {
            let test = $name::from_slice(&[0u8; 32]).unwrap();
            assert!(test.as_bytes().len() == test.get_length());
            assert!(32 == test.get_length());
        }
    );
}

#[cfg(feature = "safe_api")]
/// Macro to construct a type containing sensitive data which is stored on the
/// heap.
macro_rules! construct_secret_key_variable_size {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        #[cfg(feature = "safe_api")]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name { value: Vec<u8> }

        impl_omitted_debug_trait!($name);
        impl_drop_trait!($name);
        impl_ct_partialeq_trait!($name);
        impl_default_trait!($name, $size);

        impl $name {
            func_from_slice_variable_size!($name);
            func_unprotected_as_bytes!();
            func_get_length!();
            func_generate_variable_size!($name);
        }

        #[test]
        fn test_from_slice_key() {
            assert!($name::from_slice(&[0u8; 512]).is_ok());
            assert!($name::from_slice(&[0u8; 256]).is_ok());
            assert!($name::from_slice(&[0u8; 1]).is_ok());
            assert!($name::from_slice(&[0u8; 0]).is_err());
        }

        #[test]
        fn test_unprotected_as_bytes_derived_key() {
            let test = $name::from_slice(&[0u8; 256]).unwrap();
            assert!(test.unprotected_as_bytes().len() == 256);
            assert!(test.unprotected_as_bytes() == [0u8; 256].as_ref());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_secret_key() {
            assert!($name::generate(0).is_err());
            assert!($name::generate(usize::max_value()).is_err());
            assert!($name::generate(1).is_ok());
            assert!($name::generate(64).is_ok());

            let test_zero = $name::from_slice(&[0u8; 128]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate(128).unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == 128);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_secret_key() {
            let secret = format!("{:?}", [0u8; $size].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; $size]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}

#[cfg(feature = "safe_api")]
/// Macro to construct a type containing non-sensitive which is stored on the
/// heap.
macro_rules! construct_salt_variable_size {
    ($(#[$meta:meta])*
    ($name:ident, $size:expr)) => (
        #[must_use]
        #[cfg(feature = "safe_api")]
        $(#[$meta])*
        ///
        pub struct $name { value: Vec<u8> }

        impl_normal_debug_trait!($name);
        impl_default_trait!($name, $size);
        impl_normal_partialeq_trait!($name);

        impl $name {
            func_from_slice_variable_size!($name);
            func_as_bytes!();
            func_get_length!();
            func_generate_variable_size!($name);
        }

        #[test]
        fn test_form_slice_salt() {
            assert!($name::from_slice(&[0u8; 512]).is_ok());
            assert!($name::from_slice(&[0u8; 256]).is_ok());
            assert!($name::from_slice(&[0u8; 1]).is_ok());
            assert!($name::from_slice(&[0u8; 0]).is_err());
        }

        #[test]
        fn test_as_bytes_salt() {
            let test = $name::from_slice(&[0u8; 256]).unwrap();
            assert!(test.as_bytes().len() == 256);
            assert!(test.as_bytes() == [0u8; 256].as_ref());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_salt() {
            assert!($name::generate(0).is_err());
            assert!($name::generate(usize::max_value()).is_err());
            assert!($name::generate(1).is_ok());
            assert!($name::generate(64).is_ok());

            let test_zero = $name::from_slice(&[0u8; 128]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate(128).unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == 128);
        }
    );
}

#[cfg(feature = "safe_api")]
/// Macro to construct a password on the heap.
macro_rules! construct_password_variable_size {
    ($(#[$meta:meta])*
    ($name:ident)) => (
        #[must_use]
        #[cfg(feature = "safe_api")]
        $(#[$meta])*
        ///
        /// # Security:
        /// - __**Avoid using**__ `unprotected_as_bytes()` whenever possible, as it breaks all protections
        /// that the type implements.
        pub struct $name { value: Vec<u8> }

        impl_omitted_debug_trait!($name);
        impl_drop_trait!($name);
        impl_ct_partialeq_trait!($name);

        impl $name {
            func_from_slice_variable_size!($name);
            func_unprotected_as_bytes!();
            func_get_length!();
            func_generate_variable_size!($name);
        }

        #[test]
        fn test_form_slice_password() {
            assert!($name::from_slice(&[0u8; 512]).is_ok());
            assert!($name::from_slice(&[0u8; 256]).is_ok());
            assert!($name::from_slice(&[0u8; 1]).is_ok());
            assert!($name::from_slice(&[0u8; 0]).is_err());
        }
        #[test]
        fn test_unprotected_as_bytes_password() {
            let test = $name::from_slice(&[0u8; 256]).unwrap();
            assert!(test.unprotected_as_bytes().len() == 256);
            assert!(test.unprotected_as_bytes() == [0u8; 256].as_ref());
        }

        #[test]
        #[cfg(feature = "safe_api")]
        fn test_generate_password() {
            assert!($name::generate(0).is_err());
            assert!($name::generate(usize::max_value()).is_err());
            assert!($name::generate(1).is_ok());
            assert!($name::generate(64).is_ok());

            let test_zero = $name::from_slice(&[0u8; 128]).unwrap();
            // A random one should never be all 0's.
            let test_rand = $name::generate(128).unwrap();
            assert!(test_zero != test_rand);
            // A random generated one should always be $size in length.
            assert!(test_rand.get_length() == 128);
        }

        #[test]
        #[cfg(feature = "safe_api")]
        // format! is only available with std
        fn test_omitted_debug_password() {
            let secret = format!("{:?}", [0u8; 64].as_ref());
            let test_debug_contents = format!("{:?}", $name::from_slice(&[0u8; 64]).unwrap());
            assert_eq!(test_debug_contents.contains(&secret), false);
        }
    );
}
