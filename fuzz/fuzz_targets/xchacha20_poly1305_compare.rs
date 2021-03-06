#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate orion;
extern crate sodiumoxide;
pub mod util;

use orion::hazardous::aead::xchacha20poly1305;
use sodiumoxide::crypto::aead::xchacha20poly1305_ietf;
use util::*;

fuzz_target!(|data: &[u8]| {
	sodiumoxide::init().unwrap();

	let (key, nonce, aad, plaintext) = aead_setup_with_nonce_len(24, data);
	let mut ciphertext_with_tag_orion: Vec<u8> = vec![0u8; plaintext.len() + 16];
	let mut plaintext_out_orion = vec![0u8; plaintext.len()];

	let orion_key = xchacha20poly1305::SecretKey::from_slice(&key).unwrap();
	let orion_nonce = xchacha20poly1305::Nonce::from_slice(&nonce).unwrap();

	xchacha20poly1305::seal(
		&orion_key,
		&orion_nonce,
		&plaintext,
		Some(&aad),
		&mut ciphertext_with_tag_orion,
	)
	.unwrap();
	xchacha20poly1305::open(
		&orion_key,
		&orion_nonce,
		&ciphertext_with_tag_orion,
		Some(&aad),
		&mut plaintext_out_orion,
	)
	.unwrap();

	let sodium_key = xchacha20poly1305_ietf::Key::from_slice(&key).unwrap();
	let sodium_nonce = xchacha20poly1305_ietf::Nonce::from_slice(&nonce).unwrap();
	let sodium_ct_with_tag =
		xchacha20poly1305_ietf::seal(&plaintext, Some(&aad), &sodium_nonce, &sodium_key);
	let sodium_pt =
		xchacha20poly1305_ietf::open(&sodium_ct_with_tag, Some(&aad), &sodium_nonce, &sodium_key)
			.unwrap();
	// First verify they produce same ciphertext/plaintext
	assert_eq!(sodium_ct_with_tag, ciphertext_with_tag_orion);
	assert_eq!(plaintext_out_orion, sodium_pt);
	// Then let orion decrypt sodiumoxide ciphertext, and let sodiumoxide decrypt
	// orion ciphertext
	assert!(xchacha20poly1305_ietf::open(
		&ciphertext_with_tag_orion,
		Some(&aad),
		&sodium_nonce,
		&sodium_key
	)
	.is_ok());
	assert!(xchacha20poly1305::open(
		&orion_key,
		&orion_nonce,
		&sodium_ct_with_tag,
		Some(&aad),
		&mut plaintext_out_orion,
	)
	.is_ok());
});
