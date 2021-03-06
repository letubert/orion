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

//! # Parameters:
//! - `data`:  Data to be processed.
//! - `dst_out`: Destination buffer for the digest. The length of the digest is
//!   implied by the length of `dst_out`.
//! - `name`: Optional function-name string. If `None` it is set to a
//!   zero-length string. It should be `None` in almost all cases.
//! - `custom`: Customization string.
//!
//! `custom`: "The customization string is intended to avoid a collision between
//! these two cSHAKE values—it will be very difficult for an attacker to
//! somehow force one computation (the email signature) to yield the same result
//! as the other computation (the key fingerprint) if different values of S are used." See [NIST SP 800-185](https://csrc.nist.gov/publications/detail/sp/800-185/final) for more information.
//!
//! `name`: A special parameter that in most cases should be just set to a zero
//! string. "This is intended for use by NIST in defining SHA-3-derived
//! functions, and should only be set to values defined by NIST". See [NIST SP 800-185](https://csrc.nist.gov/publications/detail/sp/800-185/final) for more information.
//!
//! ### Note:
//! The cSHAKE256 implementation currently relies on the `tiny-keccak` crate.
//! Currently, this crate will produce ***incorrect results on big-endian based systems***. See the [issue here](https://github.com/debris/tiny-keccak/issues/15).
//!
//! # Errors:
//! An error will be returned if:
//! - The length of `dst_out` is zero.
//! - The length of `dst_out` is greater than 65536.
//! - `finalize()` is called twice in a row without calling `reset()` in
//!   between.
//! - `update()` is called after `finalize()` without a `reset()` in between.
//! - Both `name` and `custom` are empty.
//! - If the length of either `name` or `custom` is greater than 65536.
//!
//! The reason that `name` and `custom` cannot both be empty is because that
//! would be equivalent to a SHAKE call.
//!
//! # Security:
//! - cSHAKE256 has a security strength of 256 bits.
//! - The recommended output length for cSHAKE256 is 64.
//!
//! # Example:
//! ```
//! use orion::hazardous::xof::cshake;
//!
//! let input = b"\x00\x01\x02\x03";
//! let custom = b"Email signature";
//! let mut out = [0u8; 64];
//!
//! let mut hash = cshake::init(custom, None).unwrap();
//! hash.update(input).unwrap();
//!
//! hash.finalize(&mut out).unwrap();
//! ```
extern crate core;

use self::core::mem;
use crate::errors::{FinalizationCryptoError, UnknownCryptoError};
use tiny_keccak::Keccak;

#[must_use]
#[derive(Clone)]
/// cSHAKE256 streaming state.
pub struct CShake {
	setup_hasher: Keccak,
	hasher: Keccak,
	is_finalized: bool,
}

impl core::fmt::Debug for CShake {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(
			f,
			"CShake {{ setup_hasher: Unknown, hasher: Unknown, is_finalized: {:?} }}",
			self.is_finalized
		)
	}
}

impl CShake {
	/// Initial setup with encoding of `custom` and `name`.
	fn setup(&mut self, custom: &[u8], name: &[u8]) -> Result<(), UnknownCryptoError> {
		if (name.is_empty()) && (custom.is_empty()) {
			return Err(UnknownCryptoError);
		}

		if name.len() > 65536 || custom.len() > 65536 {
			return Err(UnknownCryptoError);
		}

		// Only append the left encoded rate, not the rate itself as with `name` and
		// `custom`
		let (encoded, offset) = left_encode(136_u64);
		self.hasher.update(&encoded[(offset - 1)..]);

		// The below two calls are equivalent to encode_string() from the spec
		let (encoded, offset) = left_encode(name.len() as u64 * 8);
		self.hasher.update(&encoded[(offset - 1)..]);
		self.hasher.update(&name);

		let (encoded, offset) = left_encode(custom.len() as u64 * 8);
		self.hasher.update(&encoded[(offset - 1)..]);
		self.hasher.update(custom);

		// Pad with zeroes before calling pad() in finalize()
		self.hasher.fill_block();
		self.setup_hasher = self.hasher.clone();

		Ok(())
	}

	/// Reset to `init()` state.
	pub fn reset(&mut self) {
		self.hasher = self.setup_hasher.clone();
		self.is_finalized = false;
	}

	#[must_use]
	/// Set `data`. Can be called repeatedly.
	pub fn update(&mut self, data: &[u8]) -> Result<(), FinalizationCryptoError> {
		if self.is_finalized {
			Err(FinalizationCryptoError)
		} else {
			self.hasher.update(data);
			Ok(())
		}
	}

	#[must_use]
	/// Return a cSHAKE hash and copy into `dst_out`.
	pub fn finalize(&mut self, dst_out: &mut [u8]) -> Result<(), FinalizationCryptoError> {
		if self.is_finalized {
			return Err(FinalizationCryptoError);
		}

		self.is_finalized = true;

		if dst_out.is_empty() || (dst_out.len() > 65536) {
			return Err(FinalizationCryptoError);
		}

		let mut hasher_new = Keccak::new(136, 0x04);
		mem::swap(&mut self.hasher, &mut hasher_new);

		hasher_new.finalize(dst_out);

		Ok(())
	}
}

#[must_use]
/// Initialize a `CShake` struct.
pub fn init(custom: &[u8], name: Option<&[u8]>) -> Result<CShake, UnknownCryptoError> {
	// "When N and S are both empty strings, cSHAKE(X, L, N, S) is equivalent to
	// SHAKE as defined in FIPS 202"
	let name_val = match name {
		Some(ref n_val) => *n_val,
		None => &[0u8; 0],
	};

	// 136 is the rate of Keccak512
	let mut hash = CShake {
		setup_hasher: Keccak::new(136, 0x04),
		hasher: Keccak::new(136, 0x04),
		is_finalized: false,
	};

	hash.setup(custom, name_val)?;

	Ok(hash)
}

#[must_use]
/// The left_encode function as specified in the NIST SP 800-185.
fn left_encode(x: u64) -> ([u8; 9], usize) {
	let mut input = [0u8; 9];
	let offset: usize = if x == 0 {
		8
	} else {
		let mut tmp: usize = 0;
		input[1..].copy_from_slice(&x.to_be_bytes());
		for idx in &input {
			if *idx != 0 {
				break;
			}
			tmp += 1;
		}

		tmp
	};

	input[offset - 1] = (9 - offset) as u8;

	(input, offset)
}

// Testing public functions in the module.
#[cfg(test)]
mod public {
	use super::*;

	mod test_endianness_issue {
		use super::*;
		// See: https://github.com/brycx/orion/issues/15
		#[test]
		#[cfg(target_endian = "little")]
		fn non_8_div_len() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"Email Signature";
			let mut out = [0u8; 17];

			let mut cshake = init(custom, None).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();

			let expected = b"\xD0\x08\x82\x8E\x2B\x80\xAC\x9D\x22\x18\xFF\xEE\x1D\x07\x0C\x48\xB8\
                        \xE4\xC8\x7B\xFF\x32\xC9\x69\x9D\x5B\x68\x96\xEE\xE0\xED\xD1\x64\x02\
                        \x0E\x2B\xE0\x56\x08\x58\xD9\xC0\x0C\x03\x7E\x34\xA9\x69\x37\xC5\x61\
                        \xA7\x4C\x41\x2B\xB4\xC7\x46\x46\x95\x27\x28\x1C\x8C";

			assert_eq!(expected[..17].len(), out.len());
			assert_eq!(out, &expected[..17]);
		}

		// See: https://github.com/brycx/orion/issues/15
		#[test]
		#[cfg(target_endian = "little")]
		fn result_ok() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, None).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();

			let expected = b"\xD0\x08\x82\x8E\x2B\x80\xAC\x9D\x22\x18\xFF\xEE\x1D\x07\x0C\x48\xB8\
                        \xE4\xC8\x7B\xFF\x32\xC9\x69\x9D\x5B\x68\x96\xEE\xE0\xED\xD1\x64\x02\
                        \x0E\x2B\xE0\x56\x08\x58\xD9\xC0\x0C\x03\x7E\x34\xA9\x69\x37\xC5\x61\
                        \xA7\x4C\x41\x2B\xB4\xC7\x46\x46\x95\x27\x28\x1C\x8C";

			assert_eq!(out.as_ref(), expected.as_ref());
		}

		// See: https://github.com/brycx/orion/issues/15
		// Detecting test-case that if tiny-keccak is fixed then this should panic
		#[test]
		#[cfg(target_endian = "big")]
		fn result_ok_assume_wrong_on_big_endian() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, None).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();

			let expected = b"\xD0\x08\x82\x8E\x2B\x80\xAC\x9D\x22\x18\xFF\xEE\x1D\x07\x0C\x48\xB8\
						\xE4\xC8\x7B\xFF\x32\xC9\x69\x9D\x5B\x68\x96\xEE\xE0\xED\xD1\x64\x02\
						\x0E\x2B\xE0\x56\x08\x58\xD9\xC0\x0C\x03\x7E\x34\xA9\x69\x37\xC5\x61\
						\xA7\x4C\x41\x2B\xB4\xC7\x46\x46\x95\x27\x28\x1C\x8C";

			assert_ne!(out.as_ref(), expected.as_ref());
		}

		// See: https://github.com/brycx/orion/issues/15
		#[test]
		#[cfg(target_endian = "little")]
		fn verify_err() {
			// `name` and `custom` values have been switched here compared to the previous
			// one
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();

			let expected = b"\xD0\x08\x82\x8E\x2B\x80\xAC\x9D\x22\x18\xFF\xEE\x1D\x07\x0C\x48\xB8\
                        \xE4\xC8\x7B\xFF\x32\xC9\x69\x9D\x5B\x68\x96\xEE\xE0\xED\xD1\x64\x02\
                        \x0E\x2B\xE0\x56\x08\x58\xD9\xC0\x0C\x03\x7E\x34\xA9\x69\x37\xC5\x61\
                        \xA7\x4C\x41\x2B\xB4\xC7\x46\x46\x95\x27\x28\x1C\x8C";

			assert_ne!(out.as_ref(), expected.as_ref());
		}
	}

	mod test_init {
		use super::*;

		#[test]
		fn err_on_empty_name_custom() {
			let custom = b"";
			let name = b"";

			assert!(init(custom, Some(name)).is_err());
		}

		#[test]
		fn empty_custom_ok() {
			let custom = b"";
			let name = b"Email signature";

			assert!(init(custom, Some(name)).is_ok());
		}

		#[test]
		fn empty_input_ok() {
			let custom = b"Custom String";
			let name = b"Email signature";

			assert!(init(custom, Some(name)).is_ok());
		}

		#[test]
		fn err_on_name_max_length() {
			let custom = b"";
			let name = [0u8; 65537];

			assert!(init(custom, Some(&name)).is_err());
		}

		#[test]
		fn err_on_n_c_max_length() {
			let custom = [0u8; 65537];
			let name = [0u8; 65537];

			assert!(init(&custom, Some(&name)).is_err());
		}

		#[test]
		fn err_on_custom_max_length() {
			let custom = [0u8; 65537];
			let name = [0u8; 0];

			assert!(init(&custom, Some(&name)).is_err());
			assert!(init(&custom, None).is_err());
		}
	}

	mod test_reset {
		use super::*;

		#[test]
		fn double_reset_ok() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			cshake.reset();
			cshake.reset();
		}
	}

	mod test_update {
		use super::*;

		#[test]
		/// Related bug: https://github.com/brycx/orion/issues/28
		fn update_after_finalize_err() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			assert!(cshake.update(input).is_err());
		}

		#[test]
		fn update_after_finalize_with_reset_ok() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];
			let mut out_check = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			cshake.reset();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out_check).unwrap();

			assert_eq!(out.as_ref(), out_check.as_ref());
		}
	}

	mod test_finalize {
		use super::*;

		#[test]
		fn err_on_zero_length() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email signature";
			let mut out = [0u8; 0];

			let mut hash = init(custom, Some(name)).unwrap();
			hash.update(input).unwrap();
			assert!(hash.finalize(&mut out).is_err());
		}

		#[test]
		fn err_on_above_max_length() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email signature";
			let mut out = [0u8; 65537];

			let mut hash = init(custom, Some(name)).unwrap();
			hash.update(input).unwrap();
			assert!(hash.finalize(&mut out).is_err());
		}

		#[test]
		fn double_finalize_err() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			assert!(cshake.finalize(&mut out).is_err());
		}

		#[test]
		fn double_finalize_with_reset_ok() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			cshake.reset();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
		}

		#[test]
		fn double_finalize_with_reset_no_update_ok() {
			let input = b"\x00\x01\x02\x03";
			let custom = b"";
			let name = b"Email Signature";
			let mut out = [0u8; 64];

			let mut cshake = init(custom, Some(name)).unwrap();
			cshake.update(input).unwrap();
			cshake.finalize(&mut out).unwrap();
			cshake.reset();
			cshake.finalize(&mut out).unwrap();
		}

	}

	#[cfg(feature = "safe_api")]
	// Mark safe_api because currently it only contains proptests and tests that
	// need vec![].
	mod test_streaming_interface {
		use super::*;

		#[cfg(feature = "safe_api")]
		/// Related bug: https://github.com/brycx/orion/issues/46
		/// Testing different usage combinations of init(), update(),
		/// finalize() and reset() produce the same output.
		/// Cannot be on no_std due to use of Vec.
		fn produces_same_out(input: &[u8], custom: &[u8], name: &[u8], outsize: usize) {
			let custom_checked = if custom.is_empty() && name.is_empty() {
				b"Hello world"
			} else {
				custom
			};

			let outsize_checked = if outsize == 0 || outsize > 65536 {
				64
			} else {
				outsize
			};

			// init(), update(), finalize()
			let mut state_1 = init(custom_checked, Some(name)).unwrap();
			state_1.update(input).unwrap();
			let mut res_1 = vec![0u8; outsize_checked];
			state_1.finalize(&mut res_1).unwrap();

			// init(), reset(), update(), finalize()
			let mut state_2 = init(custom_checked, Some(name)).unwrap();
			state_2.reset();
			state_2.update(input).unwrap();
			let mut res_2 = vec![0u8; outsize_checked];
			state_2.finalize(&mut res_2).unwrap();

			// init(), update(), reset(), update(), finalize()
			let mut state_3 = init(custom_checked, Some(name)).unwrap();
			state_3.update(input).unwrap();
			state_3.reset();
			state_3.update(input).unwrap();
			let mut res_3 = vec![0u8; outsize_checked];
			state_3.finalize(&mut res_3).unwrap();

			// init(), update(), finalize(), reset(), update(), finalize()
			let mut state_4 = init(custom_checked, Some(name)).unwrap();
			state_4.update(input).unwrap();
			let mut res_4 = vec![0u8; outsize_checked];
			state_4.finalize(&mut res_4).unwrap();
			state_4.reset();
			state_4.update(input).unwrap();
			state_4.finalize(&mut res_4).unwrap();

			assert_eq!(res_1, res_2);
			assert_eq!(res_2, res_3);
			assert_eq!(res_3, res_4);
		}

		#[test]
		#[cfg(feature = "safe_api")]
		// Test for issues when incrementally processing data.
		fn test_streaming_consistency() {
			for len in 0..128 * 4 {
				let input = vec![0u8; len];
				let custom = b"Email Signature";
				let mut out = vec![0u8; len + 1];

				let mut state = init(custom, None).unwrap();
				let mut other_data: Vec<u8> = Vec::new();

				other_data.extend_from_slice(&input);
				state.update(&input).unwrap();

				if input.len() > 128 {
					other_data.extend_from_slice(b"");
					state.update(b"").unwrap();
				}
				if input.len() > 128 * 2 {
					other_data.extend_from_slice(b"Extra");
					state.update(b"Extra").unwrap();
				}
				if input.len() > 128 * 3 {
					other_data.extend_from_slice(&[0u8; 256]);
					state.update(&[0u8; 256]).unwrap();
				}

				state.finalize(&mut out).unwrap();

				let mut non_incremental = init(custom, None).unwrap();
				non_incremental.update(&other_data[..]).unwrap();
				let mut out_non_incremental = vec![0u8; len + 1];
				non_incremental.finalize(&mut out_non_incremental).unwrap();

				assert_eq!(out, out_non_incremental);
			}
		}
		// Proptests. Only exectued when NOT testing no_std.
		#[cfg(feature = "safe_api")]
		mod proptest {
			use super::*;

			quickcheck! {
				/// Related bug: https://github.com/brycx/orion/issues/46
				/// Test different streaming state usage patterns.
				fn prop_same_result_different_usage(input: Vec<u8>, custom: Vec<u8>, name: Vec<u8>, outsize: usize) -> bool {
					// Will panic on incorrect results.
					produces_same_out(&input[..], &custom[..], &name[..], outsize);

					true
				}
			}
		}
	}
}

// Testing private functions in the module.
#[cfg(test)]
mod private {
	use super::*;

	// One function tested per submodule.

	mod test_left_encode {
		use super::*;

		#[test]
		fn test_left_encode() {
			let (test_1, offset_1) = left_encode(32);
			let (test_2, offset_2) = left_encode(255);
			let (test_3, offset_3) = left_encode(0);
			let (test_4, offset_4) = left_encode(64);
			let (test_5, offset_5) = left_encode(u64::max_value());

			assert_eq!(&test_1[(offset_1 - 1)..], &[1, 32]);
			assert_eq!(&test_2[(offset_2 - 1)..], &[1, 255]);
			assert_eq!(&test_3[(offset_3 - 1)..], &[1, 0]);
			assert_eq!(&test_4[(offset_4 - 1)..], &[1, 64]);
			assert_eq!(
				&test_5[(offset_5 - 1)..],
				&[8, 255, 255, 255, 255, 255, 255, 255, 255]
			);
		}
	}
}
