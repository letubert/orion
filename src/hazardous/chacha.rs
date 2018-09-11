// MIT License

// Copyright (c) 2018 brycx

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use byteorder::{ByteOrder, LittleEndian};
use hazardous::constants::ChaChaState;
use seckey::zero;
use utilities::errors::UnknownCryptoError;

#[derive(Clone)]
struct InternalState {
    buffer: ChaChaState,
}

impl Drop for InternalState {
    fn drop(&mut self) {
        zero(&mut self.buffer)
    }
}

impl InternalState {
    /// Perform a single round on index `x`, `y` and `z` with an `n_bit_rotation` left-rotation.
    fn round(&mut self, x: usize, y: usize, z: usize, n_bit_rotation: u32) {
        self.buffer[x] = self.buffer[x].wrapping_add(self.buffer[z]);
        self.buffer[y] ^= self.buffer[x];
        self.buffer[y] = self.buffer[y].rotate_left(n_bit_rotation);
    }
    /// ChaCha quarter round on a `InternalState`. Indexed by four `usize`s.
    fn quarter_round(&mut self, x: usize, y: usize, z: usize, w: usize) {
        self.round(x, w, y, 16);
        self.round(z, y, w, 12);
        self.round(x, w, y, 8);
        self.round(z, y, w, 7);
    }
    /// Performs 8 `quarter_round` function calls to process a inner block.
    fn process_inner_block(&mut self) {
        // Perform column rounds
        self.quarter_round(0, 4, 8, 12);
        self.quarter_round(1, 5, 9, 13);
        self.quarter_round(2, 6, 10, 14);
        self.quarter_round(3, 7, 11, 15);
        // Perform diagonal rounds
        self.quarter_round(0, 5, 10, 15);
        self.quarter_round(1, 6, 11, 12);
        self.quarter_round(2, 7, 8, 13);
        self.quarter_round(3, 4, 9, 14);
    }

    fn init_chacha20_state(
        &mut self,
        key: &[u8],
        nonce: &[u8],
        block_count: u32,
    ) -> Result<(), UnknownCryptoError> {
        if !(key.len() == 32) {
            return Err(UnknownCryptoError);
        }
        if !(nonce.len() == 12) {
            return Err(UnknownCryptoError);
        }

        // Init state with four constants
        self.buffer = [
            0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        // Split key into little-endian 4-byte chunks
        for (idx_count, key_chunk) in key.chunks(4).enumerate() {
            // Indexing starts from the 4th word in the ChaCha20 state
            assert!(idx_count + 1 < 12);
            self.buffer[idx_count + 4] = LittleEndian::read_u32(&key_chunk);
        }

        self.buffer[12] = block_count.to_le();

        self.buffer[13] = LittleEndian::read_u32(&nonce[..4]);
        self.buffer[14] = LittleEndian::read_u32(&nonce[4..8]);
        self.buffer[15] = LittleEndian::read_u32(&nonce[8..12]);

        Ok(())
    }
    /// The ChaCha20 block function. Returns a single block.
    fn chacha20_block(&mut self, key: &[u8], nonce: &[u8], block_count: u32) {
        self.init_chacha20_state(key, nonce, block_count).unwrap();
        let original_state: InternalState = self.clone();

        for _ in 0..10 {
            self.process_inner_block();
        }

        for (idx, word) in self.buffer.iter_mut().enumerate() {
            *word = word.wrapping_add(original_state.buffer[idx]);
        }
    }
    /// Serialize a ChaCha20 block into an byte array.
    fn serialize_state(&mut self, dst_block: &mut [u8]) -> Result<(), UnknownCryptoError> {
        if !(dst_block.len() == 64) {
            return Err(UnknownCryptoError);
        }

        for (word, dst_byte) in self.buffer.iter().zip(dst_block.chunks_mut(4)) {
            LittleEndian::write_u32_into(&[*word], dst_byte);
        }

        Ok(())
    }
}

/// The ChaCha20 encryption function.
pub fn chacha20_encrypt(
    key: &[u8],
    nonce: &[u8],
    initial_counter: u32,
    plaintext: &[u8],
    dst_ciphertext: &mut [u8],
) -> Result<(), UnknownCryptoError> {
    if plaintext.len() != dst_ciphertext.len() {
        return Err(UnknownCryptoError);
    }

    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let mut keystream_block = [0u8; 64];

    for (counter, (plaintext_block, ciphertext_block)) in plaintext
        .chunks(64)
        .zip(dst_ciphertext.chunks_mut(64))
        .enumerate()
    {
        let chunk_counter = initial_counter.checked_add(counter as u32).unwrap();
        chacha_state.chacha20_block(key, nonce, chunk_counter);
        chacha_state.serialize_state(&mut keystream_block).unwrap();
        debug_assert!(keystream_block.len() >= plaintext_block.len());
        for (idx, itm) in plaintext_block.iter().enumerate() {
            keystream_block[idx] ^= *itm;
        }
        // `ct_chunk` and `pt_chunk` have the same length so indexing is no problem here
        ciphertext_block.copy_from_slice(&keystream_block[..plaintext_block.len()]);
    }

    zero(&mut keystream_block);

    Ok(())
}

/// The ChaCha20 decryption function.
pub fn chacha20_decrypt(
    key: &[u8],
    nonce: &[u8],
    initial_counter: u32,
    ciphertext: &[u8],
    dst_plaintext: &mut [u8],
) -> Result<(), UnknownCryptoError> {
    chacha20_encrypt(key, nonce, initial_counter, ciphertext, dst_plaintext)
}

#[test]
// From https://tools.ietf.org/html/rfc7539#section-2.1
fn test_quarter_round_results() {
    let mut chacha_state = InternalState {
        buffer: [
            0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567, 0x11111111, 0x01020304, 0x9b8d6f43,
            0x01234567, 0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567, 0x11111111, 0x01020304,
            0x9b8d6f43, 0x01234567,
        ],
    };
    let expected: [u32; 4] = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb];
    // Test all indexes
    chacha_state.quarter_round(0, 1, 2, 3);
    chacha_state.quarter_round(4, 5, 6, 7);
    chacha_state.quarter_round(8, 9, 10, 11);
    chacha_state.quarter_round(12, 13, 14, 15);

    assert_eq!(chacha_state.buffer[0..4], expected);
    assert_eq!(chacha_state.buffer[4..8], expected);
    assert_eq!(chacha_state.buffer[8..12], expected);
    assert_eq!(chacha_state.buffer[12..16], expected);
}

#[test]
// From https://tools.ietf.org/html/rfc7539#section-2.1
fn test_quarter_round_results_on_indices() {
    let mut chacha_state = InternalState {
        buffer: [
            0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
            0x2a5f714c, 0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0x3d631689,
            0x2098d9d6, 0x91dbd320,
        ],
    };
    let expected: ChaChaState = [
        0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b,
        0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79,
        0x2098d9d6, 0x91dbd320,
    ];

    chacha_state.quarter_round(2, 7, 8, 13);

    assert_eq!(chacha_state.buffer[..], expected);
}

#[test]
// From https://tools.ietf.org/html/rfc7539#section-2.1
fn test_chacha20_block_results() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
        0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
        0x1e, 0x1f,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
    ];
    let expected = [
        0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71,
        0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4,
        0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05, 0xd9,
        0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, 0x83, 0xe8,
        0xa2, 0x50, 0x3c, 0x4e,
    ];
    // Test initial key-steup
    let expected_init: ChaChaState = [
        0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908,
        0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000,
        0x4a000000, 0x00000000,
    ];

    let mut test_init_key = chacha_state.clone();
    test_init_key.init_chacha20_state(&key, &nonce, 1).unwrap();
    assert_eq!(test_init_key.buffer[..], expected_init[..]);

    chacha_state.chacha20_block(&key, &nonce, 1);
    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}

#[test]
fn chacha20_block_test_1() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let expected = [
        0x76, 0xb8, 0xe0, 0xad, 0xa0, 0xf1, 0x3d, 0x90, 0x40, 0x5d, 0x6a, 0xe5, 0x53, 0x86, 0xbd,
        0x28, 0xbd, 0xd2, 0x19, 0xb8, 0xa0, 0x8d, 0xed, 0x1a, 0xa8, 0x36, 0xef, 0xcc, 0x8b, 0x77,
        0x0d, 0xc7, 0xda, 0x41, 0x59, 0x7c, 0x51, 0x57, 0x48, 0x8d, 0x77, 0x24, 0xe0, 0x3f, 0xb8,
        0xd8, 0x4a, 0x37, 0x6a, 0x43, 0xb8, 0xf4, 0x15, 0x18, 0xa1, 0x1c, 0xc3, 0x87, 0xb6, 0x69,
        0xb2, 0xee, 0x65, 0x86,
    ];
    // Unserialized state
    let expected_state: ChaChaState = [
        0xade0b876, 0x903df1a0, 0xe56a5d40, 0x28bd8653, 0xb819d2bd, 0x1aed8da0, 0xccef36a8,
        0xc70d778b, 0x7c5941da, 0x8d485751, 0x3fe02477, 0x374ad8b8, 0xf4b8436a, 0x1ca11815,
        0x69b687c3, 0x8665eeb2,
    ];

    chacha_state.chacha20_block(&key, &nonce, 0);
    assert_eq!(chacha_state.buffer[..], expected_state[..]);

    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}

#[test]
fn chacha20_block_test_2() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let expected = [
        0x9f, 0x07, 0xe7, 0xbe, 0x55, 0x51, 0x38, 0x7a, 0x98, 0xba, 0x97, 0x7c, 0x73, 0x2d, 0x08,
        0x0d, 0xcb, 0x0f, 0x29, 0xa0, 0x48, 0xe3, 0x65, 0x69, 0x12, 0xc6, 0x53, 0x3e, 0x32, 0xee,
        0x7a, 0xed, 0x29, 0xb7, 0x21, 0x76, 0x9c, 0xe6, 0x4e, 0x43, 0xd5, 0x71, 0x33, 0xb0, 0x74,
        0xd8, 0x39, 0xd5, 0x31, 0xed, 0x1f, 0x28, 0x51, 0x0a, 0xfb, 0x45, 0xac, 0xe1, 0x0a, 0x1f,
        0x4b, 0x79, 0x4d, 0x6f,
    ];
    // Unserialized state
    let expected_state: ChaChaState = [
        0xbee7079f, 0x7a385155, 0x7c97ba98, 0x0d082d73, 0xa0290fcb, 0x6965e348, 0x3e53c612,
        0xed7aee32, 0x7621b729, 0x434ee69c, 0xb03371d5, 0xd539d874, 0x281fed31, 0x45fb0a51,
        0x1f0ae1ac, 0x6f4d794b,
    ];

    chacha_state.chacha20_block(&key, &nonce, 1);
    assert_eq!(chacha_state.buffer[..], expected_state[..]);

    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}

#[test]
fn chacha20_block_test_3() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let expected = [
        0x3a, 0xeb, 0x52, 0x24, 0xec, 0xf8, 0x49, 0x92, 0x9b, 0x9d, 0x82, 0x8d, 0xb1, 0xce, 0xd4,
        0xdd, 0x83, 0x20, 0x25, 0xe8, 0x01, 0x8b, 0x81, 0x60, 0xb8, 0x22, 0x84, 0xf3, 0xc9, 0x49,
        0xaa, 0x5a, 0x8e, 0xca, 0x00, 0xbb, 0xb4, 0xa7, 0x3b, 0xda, 0xd1, 0x92, 0xb5, 0xc4, 0x2f,
        0x73, 0xf2, 0xfd, 0x4e, 0x27, 0x36, 0x44, 0xc8, 0xb3, 0x61, 0x25, 0xa6, 0x4a, 0xdd, 0xeb,
        0x00, 0x6c, 0x13, 0xa0,
    ];
    // Unserialized state
    let expected_state: ChaChaState = [
        0x2452eb3a, 0x9249f8ec, 0x8d829d9b, 0xddd4ceb1, 0xe8252083, 0x60818b01, 0xf38422b8,
        0x5aaa49c9, 0xbb00ca8e, 0xda3ba7b4, 0xc4b592d1, 0xfdf2732f, 0x4436274e, 0x2561b3c8,
        0xebdd4aa6, 0xa0136c00,
    ];

    chacha_state.chacha20_block(&key, &nonce, 1);
    assert_eq!(chacha_state.buffer[..], expected_state[..]);

    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}

#[test]
fn chacha20_block_test_4() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    let expected = [
        0x72, 0xd5, 0x4d, 0xfb, 0xf1, 0x2e, 0xc4, 0x4b, 0x36, 0x26, 0x92, 0xdf, 0x94, 0x13, 0x7f,
        0x32, 0x8f, 0xea, 0x8d, 0xa7, 0x39, 0x90, 0x26, 0x5e, 0xc1, 0xbb, 0xbe, 0xa1, 0xae, 0x9a,
        0xf0, 0xca, 0x13, 0xb2, 0x5a, 0xa2, 0x6c, 0xb4, 0xa6, 0x48, 0xcb, 0x9b, 0x9d, 0x1b, 0xe6,
        0x5b, 0x2c, 0x09, 0x24, 0xa6, 0x6c, 0x54, 0xd5, 0x45, 0xec, 0x1b, 0x73, 0x74, 0xf4, 0x87,
        0x2e, 0x99, 0xf0, 0x96,
    ];
    // Unserialized state
    let expected_state: ChaChaState = [
        0xfb4dd572, 0x4bc42ef1, 0xdf922636, 0x327f1394, 0xa78dea8f, 0x5e269039, 0xa1bebbc1,
        0xcaf09aae, 0xa25ab213, 0x48a6b46c, 0x1b9d9bcb, 0x092c5be6, 0x546ca624, 0x1bec45d5,
        0x87f47473, 0x96f0992e,
    ];

    chacha_state.chacha20_block(&key, &nonce, 2);
    assert_eq!(chacha_state.buffer[..], expected_state[..]);

    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}

#[test]
fn chacha20_block_test_5() {
    let mut chacha_state = InternalState {
        buffer: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    };

    let key = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let nonce = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02,
    ];
    let expected = [
        0xc2, 0xc6, 0x4d, 0x37, 0x8c, 0xd5, 0x36, 0x37, 0x4a, 0xe2, 0x04, 0xb9, 0xef, 0x93, 0x3f,
        0xcd, 0x1a, 0x8b, 0x22, 0x88, 0xb3, 0xdf, 0xa4, 0x96, 0x72, 0xab, 0x76, 0x5b, 0x54, 0xee,
        0x27, 0xc7, 0x8a, 0x97, 0x0e, 0x0e, 0x95, 0x5c, 0x14, 0xf3, 0xa8, 0x8e, 0x74, 0x1b, 0x97,
        0xc2, 0x86, 0xf7, 0x5f, 0x8f, 0xc2, 0x99, 0xe8, 0x14, 0x83, 0x62, 0xfa, 0x19, 0x8a, 0x39,
        0x53, 0x1b, 0xed, 0x6d,
    ];
    // Unserialized state
    let expected_state: ChaChaState = [
        0x374dc6c2, 0x3736d58c, 0xb904e24a, 0xcd3f93ef, 0x88228b1a, 0x96a4dfb3, 0x5b76ab72,
        0xc727ee54, 0x0e0e978a, 0xf3145c95, 0x1b748ea8, 0xf786c297, 0x99c28f5f, 0x628314e8,
        0x398a19fa, 0x6ded1b53,
    ];

    chacha_state.chacha20_block(&key, &nonce, 0);
    assert_eq!(chacha_state.buffer[..], expected_state[..]);

    let mut ser_block = [0u8; 64];
    chacha_state.serialize_state(&mut ser_block).unwrap();
    assert_eq!(ser_block[..], expected[..]);
}
