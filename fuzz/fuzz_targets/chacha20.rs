#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate orion;
pub mod util;

use self::util::*;
use orion::hazardous::stream::chacha20;

fuzz_target!(|data: &[u8]| {
	let (key, nonce) = chacha_key_nonce_setup(12, data);
	let mut pt = Vec::new();
	apply_from_input_heap(&mut pt, data, key.len() + nonce.len());

	let icount = data.len() as u32;
	let mut dst_pt = vec![0u8; pt.len()];
	let mut dst_ct = vec![0u8; pt.len()];

	let orion_key = chacha20::SecretKey::from_slice(&key).unwrap();
	let orion_nonce = chacha20::Nonce::from_slice(&nonce).unwrap();

	// Encrypt data
	chacha20::encrypt(&orion_key, &orion_nonce, icount, &pt, &mut dst_ct).unwrap();
	// Decrypt the ciphertext and verify it matches data
	chacha20::decrypt(&orion_key, &orion_nonce, icount, &dst_ct, &mut dst_pt).unwrap();
	assert_eq!(&dst_pt, &pt);
});
