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

/// The blocksize for the hash function SHA512.
pub const SHA512_BLOCKSIZE: usize = 128;
/// The output size for the hash function SHA512.
pub const SHA512_OUTSIZE: usize = 64;
/// The blocksize which ChaCha20 operates on.
pub const CHACHA_BLOCKSIZE: usize = 64;
/// The key size for ChaCha20.
pub const CHACHA_KEYSIZE: usize = 32;
/// The size of the subkey that HChaCha20 returns.
pub const HCHACHA_OUTSIZE: usize = 32;
/// The nonce size for IETF ChaCha20.
pub const IETF_CHACHA_NONCESIZE: usize = 12;
/// The nonce size for HChaCha20.
pub const HCHACHA_NONCESIZE: usize = 16;
/// The nonce size for XChaCha20.
pub const XCHACHA_NONCESIZE: usize = 24;
/// The blocksize which Poly1305 operates on.
pub const POLY1305_BLOCKSIZE: usize = 16;
/// The output size for Poly1305.
pub const POLY1305_OUTSIZE: usize = 16;
/// The key size for Poly1305.
pub const POLY1305_KEYSIZE: usize = 32;
/// The blocksize for the hash function BLAKE2b.
pub const BLAKE2B_BLOCKSIZE: usize = 128;
/// The key size for the hash function BLAKE2b when used in keyed mode.
pub const BLAKE2B_KEYSIZE: usize = 64;
/// The output size for the hash function BLAKE2b.
pub const BLAKE2B_OUTSIZE: usize = 64;

/// Type for an array of length `SHA512_BLOCKSIZE`.
pub type BlocksizeArray = [u8; SHA512_BLOCKSIZE];
/// Type for an array of length `SHA512_OUTSIZE`.
pub type HLenArray = [u8; SHA512_OUTSIZE];
/// Type for a ChaCha state represented as an array of 16 32-bit unsigned
/// integers.
pub type ChaChaState = [u32; 16];
/// Type for a Poly1305 tag.
pub type Poly1305Tag = [u8; POLY1305_OUTSIZE];
