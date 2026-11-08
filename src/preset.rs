use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{CableConnection, ChainMode, ModularNode, NodeType, ParametricEqPreset, StudioMode};
use crate::app::LunulaApp;

// ============================================================================
// 1. CRYPTOGRAPHIC PRIMITIVES (SHA-256, HMAC-SHA256, HKDF, ChaCha20)
// Pure Rust, constant-time, zero external crate dependencies
// ============================================================================

/// Master Lunula Studio Signature & Encryption Key
const LUNULA_MASTER_KEY: &[u8] = b"LunulaAudioPlayer::StudioPreset::MasterSecretKey::v1";

/// SHA-256 Constants
const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, mut data: &[u8]) {
        self.total_len += data.len() as u64;

        if self.buf_len > 0 {
            let take = (64 - self.buf_len).min(data.len());
            self.buffer[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            data = &data[take..];

            if self.buf_len == 64 {
                let buf = self.buffer;
                Self::process_block(&mut self.state, &buf);
                self.buf_len = 0;
            }
        }

        while data.len() >= 64 {
            let block: &[u8; 64] = data[..64].try_into().unwrap();
            Self::process_block(&mut self.state, block);
            data = &data[64..];
        }

        if !data.is_empty() {
            self.buffer[..data.len()].copy_from_slice(data);
            self.buf_len = data.len();
        }
    }

    fn process_block(state: &mut [u32; 8], block: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(block[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(K256[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    pub fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len * 8;
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            self.buffer[self.buf_len..].fill(0);
            let buf = self.buffer;
            Self::process_block(&mut self.state, &buf);
            self.buffer.fill(0);
        } else {
            self.buffer[self.buf_len..56].fill(0);
        }

        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let buf = self.buffer;
        Self::process_block(&mut self.state, &buf);

        let mut out = [0u8; 32];
        for (i, word) in self.state.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// HMAC-SHA256 (RFC 2104)
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hash = sha256(key);
        k[..32].copy_from_slice(&hash);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut ipad = [0u8; 64];
    let mut opad = [0u8; 64];
    for i in 0..64 {
        ipad[i] = k[i] ^ 0x36;
        opad[i] = k[i] ^ 0x5c;
    }

    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(message);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize()
}

/// HKDF-SHA256 (RFC 5869)
pub fn hkdf_sha256(salt: &[u8], ikm: &[u8], info: &[u8], okm: &mut [u8]) {
    // Extract: PRK = HMAC-Hash(salt, IKM)
    let prk = hmac_sha256(salt, ikm);

    // Expand
    let mut t = Vec::new();
    let mut offset = 0;
    let mut i = 1u8;

    while offset < okm.len() {
        let mut msg = Vec::new();
        msg.extend_from_slice(&t);
        msg.extend_from_slice(info);
        msg.push(i);

        let step_hash = hmac_sha256(&prk, &msg);
        t = step_hash.to_vec();

        let to_copy = (okm.len() - offset).min(32);
        okm[offset..offset + to_copy].copy_from_slice(&step_hash[..to_copy]);
        offset += to_copy;
        i += 1;
    }
}

/// ChaCha20 Stream Cipher (RFC 8439)
pub fn chacha20_xor(key: &[u8; 32], nonce: &[u8; 12], counter: u32, data: &mut [u8]) {
    #[inline(always)]
    fn qr(w: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
        w[a] = w[a].wrapping_add(w[b]); w[d] ^= w[a]; w[d] = w[d].rotate_left(16);
        w[c] = w[c].wrapping_add(w[d]); w[b] ^= w[c]; w[b] = w[b].rotate_left(12);
        w[a] = w[a].wrapping_add(w[b]); w[d] ^= w[a]; w[d] = w[d].rotate_left(8);
        w[c] = w[c].wrapping_add(w[d]); w[b] ^= w[c]; w[b] = w[b].rotate_left(7);
    }

    let key_words = [
        u32::from_le_bytes(key[0..4].try_into().unwrap()),
        u32::from_le_bytes(key[4..8].try_into().unwrap()),
        u32::from_le_bytes(key[8..12].try_into().unwrap()),
        u32::from_le_bytes(key[12..16].try_into().unwrap()),
        u32::from_le_bytes(key[16..20].try_into().unwrap()),
        u32::from_le_bytes(key[20..24].try_into().unwrap()),
        u32::from_le_bytes(key[24..28].try_into().unwrap()),
        u32::from_le_bytes(key[28..32].try_into().unwrap()),
    ];

    let nonce_words = [
        u32::from_le_bytes(nonce[0..4].try_into().unwrap()),
        u32::from_le_bytes(nonce[4..8].try_into().unwrap()),
        u32::from_le_bytes(nonce[8..12].try_into().unwrap()),
    ];

    let mut current_counter = counter;
    let mut offset = 0;

    while offset < data.len() {
        let state = [
            0x61707865, 0x3320646e, 0x79622d32, 0x6b206574,
            key_words[0], key_words[1], key_words[2], key_words[3],
            key_words[4], key_words[5], key_words[6], key_words[7],
            current_counter, nonce_words[0], nonce_words[1], nonce_words[2],
        ];

        let mut working = state;
        for _ in 0..10 {
            // Column round
            qr(&mut working, 0, 4, 8, 12);
            qr(&mut working, 1, 5, 9, 13);
            qr(&mut working, 2, 6, 10, 14);
            qr(&mut working, 3, 7, 11, 15);

            // Diagonal round
            qr(&mut working, 0, 5, 10, 15);
            qr(&mut working, 1, 6, 11, 12);
            qr(&mut working, 2, 7, 8, 13);
            qr(&mut working, 3, 4, 9, 14);
        }

        for i in 0..16 {
            working[i] = working[i].wrapping_add(state[i]);
        }

        let mut keystream = [0u8; 64];
        for i in 0..16 {
            keystream[i * 4..i * 4 + 4].copy_from_slice(&working[i].to_le_bytes());
        }

        let chunk_size = (data.len() - offset).min(64);
        for i in 0..chunk_size {
            data[offset + i] ^= keystream[i];
        }

        offset += chunk_size;
        current_counter = current_counter.wrapping_add(1);
    }
}

/// Constant-time memory comparison to protect against timing attacks
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut res = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        res |= x ^ y;
    }
    res == 0
}

/// Hex encoding
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

/// Hex decoding
pub fn hex_decode(s: &str) -> Result<Vec<u8>, PresetError> {
    let clean = s.trim();
    if clean.len() % 2 != 0 {
        return Err(PresetError::FormatError("Hex string length must be even".into()));
    }
    let mut bytes = Vec::with_capacity(clean.len() / 2);
    let chars = clean.as_bytes();
    for i in (0..chars.len()).step_by(2) {
        let hi = match chars[i] {
            b'0'..=b'9' => chars[i] - b'0',
            b'a'..=b'f' => chars[i] - b'a' + 10,
            b'A'..=b'F' => chars[i] - b'A' + 10,
            _ => return Err(PresetError::FormatError(format!("Invalid hex char '{}'", chars[i] as char))),
        };
        let lo = match chars[i + 1] {
            b'0'..=b'9' => chars[i + 1] - b'0',
            b'a'..=b'f' => chars[i + 1] - b'a' + 10,
            b'A'..=b'F' => chars[i + 1] - b'A' + 10,
            _ => return Err(PresetError::FormatError(format!("Invalid hex char '{}'", chars[i + 1] as char))),
        };
        bytes.push((hi << 4) | lo);
    }
    Ok(bytes)
}

/// Standard Base64 Encoding
pub fn base64_encode(data: &[u8]) -> String {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(B64[(b0 >> 2) as usize] as char);
        out.push(B64[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Standard Base64 Decoding
pub fn base64_decode(s: &str) -> Result<Vec<u8>, PresetError> {
    fn char_to_val(c: u8) -> Result<u8, PresetError> {
        match c {
            b'A'..=b'Z' => Ok(c - b'A'),
            b'a'..=b'z' => Ok(c - b'a' + 26),
            b'0'..=b'9' => Ok(c - b'0' + 52),
            b'+' => Ok(62),
            b'/' => Ok(63),
            _ => Err(PresetError::FormatError(format!("Invalid base64 character '{}'", c as char))),
        }
    }

    let clean: Vec<u8> = s.bytes().filter(|&b| !b.is_ascii_whitespace()).collect();
    if clean.len() % 4 != 0 {
        return Err(PresetError::FormatError("Base64 string length must be multiple of 4".into()));
    }

    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let c0 = char_to_val(chunk[0])?;
        let c1 = char_to_val(chunk[1])?;
        out.push((c0 << 2) | (c1 >> 4));

        if chunk[2] != b'=' {
            let c2 = char_to_val(chunk[2])?;
            out.push(((c1 & 0x0f) << 4) | (c2 >> 2));
            if chunk[3] != b'=' {
                let c3 = char_to_val(chunk[3])?;
                out.push(((c2 & 0x03) << 6) | c3);
            }
        }
    }
    Ok(out)
}

/// Generates pseudo-random entropy for salt and nonce using clock and PRNG state
pub fn generate_crypto_entropy(len: usize) -> Vec<u8> {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(123456789);

    let mut seed = (nanos as u64) ^ 0xa0761d6478bd642f;
    let mut out = Vec::with_capacity(len);

    for i in 0..len {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let b = ((seed >> (i % 8 * 8)) & 0xff) as u8;
        out.push(b);
    }
    out
}

// ============================================================================
// 2. PRESET DATA STRUCTURES & ERROR TYPES
// ============================================================================

#[derive(Debug)]
pub enum PresetError {
    FormatError(String),
    SerializationError(String),
    InvalidMagic,
    InvalidSignature,
    DecryptionFailed,
    IoError(String),
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresetError::FormatError(msg) => write!(f, "Format Error: {}", msg),
            PresetError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            PresetError::InvalidMagic => write!(f, "Berkas bukan preset Lunula Studio yang valid (.lnl)"),
            PresetError::InvalidSignature => write!(f, "Verifikasi tanda tangan digital gagal: Berkas preset telah dimodifikasi atau password salah!"),
            PresetError::DecryptionFailed => write!(f, "Dekripsi data sinyal studio gagal"),
            PresetError::IoError(msg) => write!(f, "I/O Error: {}", msg),
        }
    }
}

impl std::error::Error for PresetError {}

/// Serialized representation of a Modular Node in the Signal Chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetNode {
    pub id: usize,
    pub node_type: NodeType,
    pub title: String,
    pub sub1: String,
    pub sub2: String,
    pub pos: [f32; 2],
    pub is_removable: bool,
    pub param_val: f32,
}

/// The inner, decrypted payload representing the full Signal Chain Studio setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalChainPreset {
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: u64,
    pub app_version: String,
    pub chain_mode: ChainMode,
    pub studio_mode: StudioMode,
    pub nodes: Vec<PresetNode>,
    pub connections: Vec<CableConnection>,
    pub active_eq: Option<ParametricEqPreset>,
    #[serde(default)]
    pub advance_studio_dither_mode: usize,
}

/// Outer metadata visible without decrypting the payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnlPresetMetadata {
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: u64,
    pub app_version: String,
    pub node_count: usize,
    pub connection_count: usize,
    pub has_password: bool,
}

/// Cryptographic authentication and integrity header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnlCryptoHeader {
    pub algorithm: String,     // "CHACHA20-HMAC-SHA256"
    pub salt_hex: String,      // 16-byte salt
    pub nonce_hex: String,     // 12-byte nonce
    pub signature_hex: String, // HMAC-SHA256 signature
}

/// The complete `.lnl` JSON container file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LnlPresetContainer {
    pub format: String, // "LUNULA_STUDIO_PRESET"
    pub version: u32,   // 1
    pub magic: String,   // "LNLPRESET"
    pub metadata: LnlPresetMetadata,
    pub crypto: LnlCryptoHeader,
    pub payload_base64: String, // ChaCha20 encrypted ciphertext in base64
}

/// Target scope for automatic preset binding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresetBindingType {
    Track,
    Album,
    Genre,
}

impl PresetBindingType {
    pub fn weight(&self) -> u32 {
        match self {
            PresetBindingType::Track => 300,
            PresetBindingType::Album => 200,
            PresetBindingType::Genre => 100,
        }
    }

    pub fn weight_badge(&self) -> &'static str {
        match self {
            PresetBindingType::Track => "Weight 300",
            PresetBindingType::Album => "Weight 200",
            PresetBindingType::Genre => "Weight 100",
        }
    }

    pub fn badge_label(&self) -> &'static str {
        match self {
            PresetBindingType::Track => "🎵 TREK",
            PresetBindingType::Album => "💿 ALBUM",
            PresetBindingType::Genre => "🎭 GENRE",
        }
    }

    pub fn precedence_note(&self) -> &'static str {
        match self {
            PresetBindingType::Track => "Override Mutlak: Mengabaikan preset Album & Genre untuk trek ini",
            PresetBindingType::Album => "Berlaku untuk semua trek dalam album ini (kecuali jika trek memiliki preset individual)",
            PresetBindingType::Genre => "Preset payung untuk semua trek dengan genre ini",
        }
    }

    pub fn colors(&self) -> (Color32, Color32) {
        match self {
            PresetBindingType::Track => (Color32::from_rgb(236, 253, 245), Color32::from_rgb(16, 185, 129)),
            PresetBindingType::Album => (Color32::from_rgb(238, 242, 255), Color32::from_rgb(79, 70, 229)),
            PresetBindingType::Genre => (Color32::from_rgb(250, 245, 255), Color32::from_rgb(124, 58, 237)),
        }
    }
}

/// A binding rule connecting a specific Track, Album, or Genre to a Signal Chain Preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetBindingRule {
    pub id: String,
    pub target_type: PresetBindingType,
    pub target_key: String,
    pub target_label: String,
    pub preset_name: String,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

/// Persistent container for Auto-Binding configuration and rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBindingStorage {
    #[serde(default = "default_true")]
    pub auto_load_enabled: bool,
    #[serde(default = "default_false")]
    pub show_toast: bool,
    #[serde(default = "default_true")]
    pub fallback_to_bit_perfect: bool,
    #[serde(default)]
    pub bindings: Vec<PresetBindingRule>,
}

/// Local library preset entry for displaying and managing saved presets
#[derive(Debug, Clone)]
pub struct LocalPresetEntry {
    pub file_path: PathBuf,
    pub file_name: String,
    pub metadata: LnlPresetMetadata,
    pub cached_preset: Option<SignalChainPreset>,
}

// ============================================================================
// 3. ENCRYPTION, SIGNING & VERIFICATION ENGINE
// ============================================================================

impl SignalChainPreset {
    /// Encrypts and cryptographically signs this preset into a `.lnl` container.
    pub fn to_container(&self, passphrase: Option<&str>) -> Result<LnlPresetContainer, PresetError> {
        // 1. Serialize inner JSON
        let inner_json = serde_json::to_string(self)
            .map_err(|e| PresetError::SerializationError(e.to_string()))?;
        let mut plaintext = inner_json.into_bytes();

        // 2. Generate random 16-byte Salt and 12-byte Nonce
        let salt = generate_crypto_entropy(16);
        let nonce_bytes = generate_crypto_entropy(12);
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_bytes);

        // 3. Key Derivation via HKDF-SHA256
        let mut ikm = Vec::new();
        ikm.extend_from_slice(LUNULA_MASTER_KEY);
        if let Some(pass) = passphrase {
            if !pass.trim().is_empty() {
                ikm.push(b':');
                ikm.extend_from_slice(pass.as_bytes());
            }
        }

        let mut okm = [0u8; 64];
        hkdf_sha256(&salt, &ikm, b"Lunula::StudioPreset::EncAndAuthKeys::v1", &mut okm);

        let mut enc_key = [0u8; 32];
        let mut auth_key = [0u8; 32];
        enc_key.copy_from_slice(&okm[..32]);
        auth_key.copy_from_slice(&okm[32..64]);

        // 4. Encrypt with ChaCha20
        chacha20_xor(&enc_key, &nonce, 1, &mut plaintext);
        let payload_base64 = base64_encode(&plaintext);

        // 5. Construct Metadata
        let metadata = LnlPresetMetadata {
            name: self.name.clone(),
            author: self.author.clone(),
            description: self.description.clone(),
            created_at: self.created_at,
            app_version: self.app_version.clone(),
            node_count: self.nodes.len(),
            connection_count: self.connections.len(),
            has_password: passphrase.map_or(false, |p| !p.trim().is_empty()),
        };

        // 6. Compute HMAC-SHA256 Signature over:
        // magic || format || version || salt || nonce || payload
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(b"LNLPRESET");
        sig_input.extend_from_slice(b"LUNULA_STUDIO_PRESET");
        sig_input.extend_from_slice(&1u32.to_be_bytes());
        sig_input.extend_from_slice(&salt);
        sig_input.extend_from_slice(&nonce);
        sig_input.extend_from_slice(payload_base64.as_bytes());

        let signature = hmac_sha256(&auth_key, &sig_input);

        let crypto = LnlCryptoHeader {
            algorithm: "CHACHA20-HMAC-SHA256".to_string(),
            salt_hex: hex_encode(&salt),
            nonce_hex: hex_encode(&nonce),
            signature_hex: hex_encode(&signature),
        };

        Ok(LnlPresetContainer {
            format: "LUNULA_STUDIO_PRESET".to_string(),
            version: 1,
            magic: "LNLPRESET".to_string(),
            metadata,
            crypto,
            payload_base64,
        })
    }

    /// Exports the preset directly into a formatted `.lnl` JSON string.
    pub fn to_lnl_string(&self, passphrase: Option<&str>) -> Result<String, PresetError> {
        let container = self.to_container(passphrase)?;
        serde_json::to_string_pretty(&container)
            .map_err(|e| PresetError::SerializationError(e.to_string()))
    }
}

impl LnlPresetContainer {
    /// Verifies the digital signature and decrypts this container into a `SignalChainPreset`.
    pub fn decrypt_and_verify(&self, passphrase: Option<&str>) -> Result<SignalChainPreset, PresetError> {
        // 1. Verify Magic and Header Integrity
        if self.magic != "LNLPRESET" || self.format != "LUNULA_STUDIO_PRESET" {
            return Err(PresetError::InvalidMagic);
        }

        // 2. Decode Salt and Nonce
        let salt = hex_decode(&self.crypto.salt_hex)?;
        let nonce_bytes = hex_decode(&self.crypto.nonce_hex)?;
        if nonce_bytes.len() != 12 {
            return Err(PresetError::FormatError("Invalid nonce length in crypto header".into()));
        }
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&nonce_bytes);

        let expected_sig = hex_decode(&self.crypto.signature_hex)?;
        if expected_sig.len() != 32 {
            return Err(PresetError::FormatError("Invalid signature length in crypto header".into()));
        }

        // 3. Derive Keys via HKDF-SHA256
        let mut ikm = Vec::new();
        ikm.extend_from_slice(LUNULA_MASTER_KEY);
        if let Some(pass) = passphrase {
            if !pass.trim().is_empty() {
                ikm.push(b':');
                ikm.extend_from_slice(pass.as_bytes());
            }
        }

        let mut okm = [0u8; 64];
        hkdf_sha256(&salt, &ikm, b"Lunula::StudioPreset::EncAndAuthKeys::v1", &mut okm);

        let mut enc_key = [0u8; 32];
        let mut auth_key = [0u8; 32];
        enc_key.copy_from_slice(&okm[..32]);
        auth_key.copy_from_slice(&okm[32..64]);

        // 4. Verify Digital Signature (Authenticate-then-Decrypt)
        let mut sig_input = Vec::new();
        sig_input.extend_from_slice(self.magic.as_bytes());
        sig_input.extend_from_slice(self.format.as_bytes());
        sig_input.extend_from_slice(&self.version.to_be_bytes());
        sig_input.extend_from_slice(&salt);
        sig_input.extend_from_slice(&nonce);
        sig_input.extend_from_slice(self.payload_base64.as_bytes());

        let computed_sig = hmac_sha256(&auth_key, &sig_input);

        if !constant_time_eq(&computed_sig, &expected_sig) {
            return Err(PresetError::InvalidSignature);
        }

        // 5. Decrypt Payload
        let mut ciphertext = base64_decode(&self.payload_base64)?;
        chacha20_xor(&enc_key, &nonce, 1, &mut ciphertext);

        // 6. Deserialize inner JSON
        let preset: SignalChainPreset = serde_json::from_slice(&ciphertext)
            .map_err(|e| PresetError::SerializationError(format!("Corrupted or incompatible preset payload: {}", e)))?;

        Ok(preset)
    }

    /// Loads and verifies a preset container from a `.lnl` JSON string.
    pub fn from_lnl_string(content: &str, passphrase: Option<&str>) -> Result<SignalChainPreset, PresetError> {
        let container: LnlPresetContainer = serde_json::from_str(content)
            .map_err(|e| PresetError::FormatError(format!("Gagal membaca struktur berkas .lnl: {}", e)))?;
        container.decrypt_and_verify(passphrase)
    }
}

// ============================================================================
// 4. LUNULA APP INTEGRATION (Capture & Apply)
// ============================================================================

impl LunulaApp {
    /// Captures the current Signal Chain Studio state into an exportable preset.
    pub fn capture_current_signal_chain_preset(&self, name: String, author: String, description: String) -> SignalChainPreset {
        use std::time::SystemTime;
        let created_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let nodes = self.nodes.iter().map(|n| {
            PresetNode {
                id: n.id,
                node_type: n.node_type.clone(),
                title: n.title.clone(),
                sub1: n.sub1.clone(),
                sub2: n.sub2.clone(),
                pos: [n.pos.x, n.pos.y],
                is_removable: n.is_removable,
                param_val: n.param_val,
            }
        }).collect();

        SignalChainPreset {
            name,
            author,
            description,
            created_at,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            chain_mode: self.chain_mode,
            studio_mode: self.studio_mode,
            nodes,
            connections: self.connections.clone(),
            active_eq: Some(self.active_eq.clone()),
            advance_studio_dither_mode: self.advance_studio_state.dither_mode_idx,
        }
    }

    /// Applies a loaded `SignalChainPreset` into the live studio session.
    pub fn apply_signal_chain_preset(&mut self, preset: &SignalChainPreset) {
        // Convert PresetNodes back to ModularNodes
        self.nodes = preset.nodes.iter().map(|pn| {
            ModularNode {
                id: pn.id,
                node_type: pn.node_type.clone(),
                title: pn.title.clone(),
                sub1: pn.sub1.clone(),
                sub2: pn.sub2.clone(),
                pos: Pos2::new(pn.pos[0], pn.pos[1]),
                is_removable: pn.is_removable,
                param_val: pn.param_val,
            }
        }).collect();

        // Calculate next available node id
        let max_id = self.nodes.iter().map(|n| n.id).max().unwrap_or(5);
        self.next_node_id = max_id + 1;

        // Apply connections and modes
        self.connections = preset.connections.clone();
        self.chain_mode = preset.chain_mode;
        self.studio_mode = preset.studio_mode;
        self.advance_studio_state.dither_mode_idx = preset.advance_studio_dither_mode;

        if let Some(ref eq) = preset.active_eq {
            self.active_eq = eq.clone();
            self.sync_parametric_to_legacy_eq();
        }

        self.selected_node = None;
        self.advance_studio_state.selected_cable_idx = None;
        self.last_applied_auto_preset = Some(preset.name.clone());

        self.invalidate_chain_cache();
        self.sync_signal_chain_to_engine();
        self.save_library_state();
    }

    /// Saves preset to a file on disk
    pub fn save_preset_file(&self, path: &Path, name: String, author: String, desc: String, pass: Option<&str>) -> Result<(), PresetError> {
        let preset = self.capture_current_signal_chain_preset(name, author, desc);
        let content = preset.to_lnl_string(pass)?;
        std::fs::write(path, content).map_err(|e| PresetError::IoError(e.to_string()))
    }

    /// Loads preset from a file on disk
    pub fn load_preset_file(&mut self, path: &Path, pass: Option<&str>) -> Result<SignalChainPreset, PresetError> {
        let content = std::fs::read_to_string(path).map_err(|e| PresetError::IoError(e.to_string()))?;
        let preset = LnlPresetContainer::from_lnl_string(&content, pass)?;
        self.apply_signal_chain_preset(&preset);
        Ok(preset)
    }

    /// Triggers the native open file dialog for importing a `.lnl` preset.
    pub fn trigger_preset_import(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Pilih Berkas Preset Lunula Studio (.lnl)")
            .add_filter("Lunula Studio Preset (*.lnl)", &["lnl"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<LnlPresetContainer>(&content) {
                        Ok(container) => {
                            if container.metadata.has_password {
                                self.pending_import_container = Some(container);
                                self.preset_import_password.clear();
                                self.preset_import_error = None;
                                self.show_import_password_modal = true;
                            } else {
                                match container.decrypt_and_verify(None) {
                                    Ok(preset) => {
                                        let name = preset.name.clone();
                                        // Save or copy to local presets directory so it appears in collection
                                        let dest_path = get_presets_dir().join(format!("{}.lnl", name));
                                        if !dest_path.exists() {
                                            let _ = std::fs::copy(&path, &dest_path);
                                        }
                                        self.inspecting_preset = Some(preset);
                                        self.preset_modal_tab = 1;
                                        self.preset_selected_idx = 0;
                                        self.show_export_preset_modal = true;
                                        self.preset_status_notification = Some((
                                            format!("🔍 Berkas '{}' dimuat ke jendela preview. Periksa kanvas lalu klik 'Terapkan'.", name),
                                            true,
                                        ));
                                    }
                                    Err(e) => {
                                        self.preset_status_notification = Some((
                                            format!("❌ Gagal memuat preset: {}", e),
                                            false,
                                        ));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.preset_status_notification = Some((
                                format!("❌ Format berkas bukan .lnl yang valid: {}", e),
                                false,
                            ));
                        }
                    }
                }
                Err(e) => {
                    self.preset_status_notification = Some((
                        format!("❌ Gagal membaca berkas: {}", e),
                        false,
                    ));
                }
            }
        }
    }

    /// Evaluates auto-binding rules against a track with strict cascading priority:
    /// Priority 1: Specific Track title
    /// Priority 2: Album name
    /// Priority 3: Genre
    /// Returns: Option<(MatchedType, PresetName, TargetLabel)>
    pub fn evaluate_auto_binding(&self, track: &crate::models::Track) -> Option<(PresetBindingType, String, String)> {
        let clean_title = track.title.trim();
        let clean_album = track.album.trim();

        // 1. Priority 1: Specific Track Title
        if !clean_title.is_empty() {
            for rule in &self.preset_bindings {
                if rule.target_type == PresetBindingType::Track && rule.target_key.trim().eq_ignore_ascii_case(clean_title) {
                    return Some((PresetBindingType::Track, rule.preset_name.clone(), rule.target_label.clone()));
                }
            }
        }

        // 2. Priority 2: Album Name
        if !clean_album.is_empty() {
            for rule in &self.preset_bindings {
                if rule.target_type == PresetBindingType::Album && rule.target_key.trim().eq_ignore_ascii_case(clean_album) {
                    return Some((PresetBindingType::Album, rule.preset_name.clone(), rule.target_label.clone()));
                }
            }
        }

        // 3. Priority 3: Genre
        if let Some(ref genre) = track.genre {
            let clean_genre = genre.trim();
            if !clean_genre.is_empty() {
                for rule in &self.preset_bindings {
                    if rule.target_type == PresetBindingType::Genre && rule.target_key.trim().eq_ignore_ascii_case(clean_genre) {
                        return Some((PresetBindingType::Genre, rule.preset_name.clone(), rule.target_label.clone()));
                    }
                }
            }
        }

        None
    }

    /// Evaluates and applies the auto-binding preset for the initial track on application startup.
    /// Operates silently (no toast notification) to ensure immediate signal chain readiness.
    pub fn apply_startup_preset(&mut self) {
        if !self.auto_load_preset_on_track_change {
            return;
        }

        let track = self.current_track.clone();
        if track.title.trim().is_empty() || track.title == "Belum Ada Lagu Diputar" {
            return;
        }

        if let Some((_target_type, preset_name, _label)) = self.evaluate_auto_binding(&track) {
            let local_presets = list_local_presets();
            let entry_opt = local_presets.into_iter().find(|p| p.metadata.name == preset_name || p.file_name == preset_name);

            if let Some(entry) = entry_opt {
                let preset_to_apply = if let Some(preset) = entry.cached_preset {
                    Some(preset)
                } else if let Ok(content) = std::fs::read_to_string(&entry.file_path) {
                    if let Ok(container) = serde_json::from_str::<LnlPresetContainer>(&content) {
                        container.decrypt_and_verify(None).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(preset) = preset_to_apply {
                    self.apply_signal_chain_preset(&preset);
                    self.set_chain_mode(ChainMode::DspActive);
                    self.last_applied_auto_preset = Some(preset_name);
                    return;
                }
            }
        }

        // Lagu tanpa preset sama sekali:
        // Kembalikan modular ke default bersih dan set mode ke Bit-Perfect!
        self.reset_to_default_signal_chain();
        self.set_chain_mode(ChainMode::BitPerfect);
        self.last_applied_auto_preset = None;
    }

    /// Automatically evaluates and applies bound preset when playing a track.
    /// If no binding rules match and `binding_fallback_to_bit_perfect` is enabled,
    /// falls back to `ChainMode::BitPerfect`.
    pub fn check_and_apply_auto_binding(&mut self, track: &crate::models::Track) {
        if !self.auto_load_preset_on_track_change {
            return;
        }

        if let Some((target_type, preset_name, _label)) = self.evaluate_auto_binding(track) {
            let already_active = self.last_applied_auto_preset.as_deref() == Some(&preset_name);

            let local_presets = list_local_presets();
            let entry_opt = local_presets.into_iter().find(|p| p.metadata.name == preset_name || p.file_name == preset_name);

            if let Some(entry) = entry_opt {
                let preset_to_apply = if let Some(preset) = entry.cached_preset {
                    Some(preset)
                } else if let Ok(content) = std::fs::read_to_string(&entry.file_path) {
                    if let Ok(container) = serde_json::from_str::<LnlPresetContainer>(&content) {
                        container.decrypt_and_verify(None).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(preset) = preset_to_apply {
                    if !already_active {
                        self.apply_signal_chain_preset(&preset);
                        self.set_chain_mode(ChainMode::DspActive);
                        self.last_applied_auto_preset = Some(preset_name.clone());
                    } else if self.chain_mode != ChainMode::DspActive {
                        self.set_chain_mode(ChainMode::DspActive);
                    }
                    if self.show_binding_toast_on_change && !already_active {
                        let (icon, type_str) = match target_type {
                            PresetBindingType::Track => ("🎵", "Trek"),
                            PresetBindingType::Album => ("💿", "Album"),
                            PresetBindingType::Genre => ("🎭", "Genre"),
                        };
                        self.preset_status_notification = Some((
                            format!("{} Preset '{}' dimuat ({})", icon, preset_name, type_str),
                            true,
                        ));
                        self.preset_status_notification_time = Some(std::time::Instant::now());
                    }
                    return;
                }
            }
        }

        // Lagu tanpa preset sama sekali:
        // Kembalikan mode modular ke default bersih (bukan mewarisi preset lagu sebelumnya) dan aktifkan Bit-Perfect!
        if !self.binding_fallback_to_bit_perfect {
            return;
        }

        let was_dsp_active = self.chain_mode == ChainMode::DspActive;
        let had_preset = self.last_applied_auto_preset.is_some() || self.nodes.len() > 4;
        if had_preset || was_dsp_active {
            self.reset_to_default_signal_chain();
            self.set_chain_mode(ChainMode::BitPerfect);
            self.last_applied_auto_preset = None;
            if self.show_binding_toast_on_change {
                self.preset_status_notification = Some((
                    "🛡️ Fallback ke Bit-Perfect".to_string(),
                    true,
                ));
                self.preset_status_notification_time = Some(std::time::Instant::now());
            }
        }
    }

    /// Persists auto-binding rules and preferences to `bindings.json` in presets directory
    pub fn save_auto_bindings(&self) {
        let storage = AutoBindingStorage {
            auto_load_enabled: self.auto_load_preset_on_track_change,
            show_toast: self.show_binding_toast_on_change,
            fallback_to_bit_perfect: self.binding_fallback_to_bit_perfect,
            bindings: self.preset_bindings.clone(),
        };

        if let Ok(json) = serde_json::to_string_pretty(&storage) {
            let path = get_presets_dir().join("bindings.json");
            let _ = std::fs::write(path, json);
        }
    }

    /// Loads auto-binding rules and preferences from `bindings.json` in presets directory
    pub fn load_auto_bindings(&mut self) {
        let path = get_presets_dir().join("bindings.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(storage) = serde_json::from_str::<AutoBindingStorage>(&content) {
                    self.auto_load_preset_on_track_change = storage.auto_load_enabled;
                    self.show_binding_toast_on_change = storage.show_toast;
                    self.binding_fallback_to_bit_perfect = storage.fallback_to_bit_perfect;
                    self.preset_bindings = storage.bindings;
                }
            }
        }
    }

    /// Returns binding lists of bound track, album, and genre names for a given preset
    pub fn get_preset_binding_targets(&self, preset_name: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut tracks = Vec::new();
        let mut albums = Vec::new();
        let mut genres = Vec::new();

        for rule in &self.preset_bindings {
            if rule.preset_name.eq_ignore_ascii_case(preset_name) {
                match rule.target_type {
                    PresetBindingType::Track => tracks.push(rule.target_key.clone()),
                    PresetBindingType::Album => albums.push(rule.target_key.clone()),
                    PresetBindingType::Genre => genres.push(rule.target_key.clone()),
                }
            }
        }

        (tracks, albums, genres)
    }

    /// Duplicates a local preset file and gives it a unique name "<Original> (Salinan)"
    pub fn duplicate_local_preset(&mut self, path: &Path) -> Result<String, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Gagal membaca berkas preset: {}", e))?;
        let mut container = serde_json::from_str::<LnlPresetContainer>(&content)
            .map_err(|e| format!("Format berkas tidak valid: {}", e))?;

        let base_name = container.metadata.name.clone();
        let presets_dir = get_presets_dir();

        let mut copy_idx = 1;
        let new_name = loop {
            let candidate = if copy_idx == 1 {
                format!("{} (Salinan)", base_name)
            } else {
                format!("{} (Salinan {})", base_name, copy_idx)
            };
            let candidate_file = presets_dir.join(format!("{}.lnl", candidate));
            if !candidate_file.exists() {
                break candidate;
            }
            copy_idx += 1;
        };

        if !container.metadata.has_password {
            let mut preset = container.decrypt_and_verify(None)
                .map_err(|e| format!("Gagal verifikasi preset: {}", e))?;
            preset.name = new_name.clone();
            preset.created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let new_container = preset.to_container(None)
                .map_err(|e| format!("Gagal mengenkripsi duplikat: {}", e))?;
            let json = serde_json::to_string_pretty(&new_container)
                .map_err(|e| format!("Gagal serialisasi: {}", e))?;
            let dest = presets_dir.join(format!("{}.lnl", new_name));
            std::fs::write(&dest, json)
                .map_err(|e| format!("Gagal menulis berkas: {}", e))?;
        } else {
            container.metadata.name = new_name.clone();
            container.metadata.created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let json = serde_json::to_string_pretty(&container)
                .map_err(|e| format!("Gagal serialisasi: {}", e))?;
            let dest = presets_dir.join(format!("{}.lnl", new_name));
            std::fs::write(&dest, json)
                .map_err(|e| format!("Gagal menulis berkas: {}", e))?;
        }

        Ok(new_name)
    }
}

/// Returns the path to the directory where user presets are saved
pub fn get_presets_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("LunulaPlayer").join("presets");
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    let dir = PathBuf::from("presets");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Pre-populates the local preset folder with standard Lunula factory presets if empty
pub fn ensure_default_factory_presets() {
    let dir = get_presets_dir();
    let sample_file = dir.join("Warm Studio Master.lnl");
    if sample_file.exists() {
        return;
    }

    let warm_master = SignalChainPreset {
        name: "Warm Studio Master".to_string(),
        author: "Lunula Sound Lab".to_string(),
        description: "Vintage tube warmth with enhanced stereophonic soundstage for immersive listening.".to_string(),
        created_at: 1794163200,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        chain_mode: ChainMode::DspActive,
        studio_mode: StudioMode::Simple,
        nodes: vec![
            PresetNode {
                id: 1,
                node_type: NodeType::Source,
                title: "FILE SOURCE".to_string(),
                sub1: "Lossless Audio".to_string(),
                sub2: "Master Bitstream".to_string(),
                pos: [30.0, 80.0],
                is_removable: false,
                param_val: 0.0,
            },
            PresetNode {
                id: 5,
                node_type: NodeType::TubeWarmer,
                title: "Tube Warmer".to_string(),
                sub1: "Vintage Triode".to_string(),
                sub2: "35% Warmth".to_string(),
                pos: [320.0, 80.0],
                is_removable: true,
                param_val: 35.0,
            },
            PresetNode {
                id: 6,
                node_type: NodeType::StereoWidener,
                title: "Stereo Widener".to_string(),
                sub1: "Haas Effect".to_string(),
                sub2: "+25% Stage".to_string(),
                pos: [580.0, 80.0],
                is_removable: true,
                param_val: 25.0,
            },
            PresetNode {
                id: 4,
                node_type: NodeType::Endpoint,
                title: "DAC OUTPUT".to_string(),
                sub1: "Hardware Line".to_string(),
                sub2: "32-bit / 384kHz".to_string(),
                pos: [840.0, 80.0],
                is_removable: false,
                param_val: 0.0,
            },
        ],
        connections: vec![
            CableConnection::new(1, 5),
            CableConnection::new(5, 6),
            CableConnection::new(6, 4),
        ],
        active_eq: None,
        advance_studio_dither_mode: 0,
    };

    if let Ok(content) = warm_master.to_lnl_string(None) {
        let _ = std::fs::write(&sample_file, content);
    }

    let acoustic_file = dir.join("Acoustic Vocal Intimacy.lnl");
    let acoustic = SignalChainPreset {
        name: "Acoustic Vocal Intimacy".to_string(),
        author: "Lunula Sound Lab".to_string(),
        description: "Optical leveling compressor with subtle analog preamp saturation for natural vocal presence.".to_string(),
        created_at: 1794163300,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        chain_mode: ChainMode::DspActive,
        studio_mode: StudioMode::Simple,
        nodes: vec![
            PresetNode {
                id: 1,
                node_type: NodeType::Source,
                title: "FILE SOURCE".to_string(),
                sub1: "Lossless Audio".to_string(),
                sub2: "Master Bitstream".to_string(),
                pos: [30.0, 80.0],
                is_removable: false,
                param_val: 0.0,
            },
            PresetNode {
                id: 5,
                node_type: NodeType::OpticalLeveler,
                title: "Optical Leveler".to_string(),
                sub1: "LA-2A Opto Cell".to_string(),
                sub2: "40% Peak Red".to_string(),
                pos: [320.0, 80.0],
                is_removable: true,
                param_val: 40.0,
            },
            PresetNode {
                id: 6,
                node_type: NodeType::AnalogPreamp,
                title: "Analog Preamp".to_string(),
                sub1: "Class-A Console".to_string(),
                sub2: "+2.0 dB".to_string(),
                pos: [580.0, 80.0],
                is_removable: true,
                param_val: 2.0,
            },
            PresetNode {
                id: 4,
                node_type: NodeType::Endpoint,
                title: "DAC OUTPUT".to_string(),
                sub1: "Hardware Line".to_string(),
                sub2: "32-bit / 384kHz".to_string(),
                pos: [840.0, 80.0],
                is_removable: false,
                param_val: 0.0,
            },
        ],
        connections: vec![
            CableConnection::new(1, 5),
            CableConnection::new(5, 6),
            CableConnection::new(6, 4),
        ],
        active_eq: None,
        advance_studio_dither_mode: 0,
    };

    if let Ok(content) = acoustic.to_lnl_string(None) {
        let _ = std::fs::write(&acoustic_file, content);
    }
}

/// Lists all valid `.lnl` preset files in the local presets directory
pub fn list_local_presets() -> Vec<LocalPresetEntry> {
    ensure_default_factory_presets();
    let dir = get_presets_dir();
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("lnl")).unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(container) = serde_json::from_str::<LnlPresetContainer>(&content) {
                        let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("preset").to_string();
                        let cached_preset = if !container.metadata.has_password {
                            container.decrypt_and_verify(None).ok()
                        } else {
                            None
                        };
                        list.push(LocalPresetEntry {
                            file_path: path,
                            file_name,
                            metadata: container.metadata,
                            cached_preset,
                        });
                    }
                }
            }
        }
    }
    list.sort_by(|a, b| b.metadata.created_at.cmp(&a.metadata.created_at));
    list
}

impl LunulaApp {
    /// Renders the styled text input container with label and icon
    pub fn render_styled_text_input(ui: &mut egui::Ui, text: &mut String, hint: &str, icon: &str) {
        egui::Frame::new()
            .fill(Color32::from_rgb(248, 250, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !icon.is_empty() {
                        ui.label(egui::RichText::new(icon).size(12.0).color(Color32::from_rgb(100, 116, 139)));
                        ui.add_space(2.0);
                    }
                    ui.add(
                        egui::TextEdit::singleline(text)
                            .hint_text(hint)
                            .frame(egui::Frame::NONE)
                            .desired_width(ui.available_width()),
                    );
                });
            });
    }

    /// Renders the styled password input container
    pub fn render_styled_password_input(ui: &mut egui::Ui, text: &mut String, hint: &str) {
        egui::Frame::new()
            .fill(Color32::from_rgb(248, 250, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔑").size(12.0));
                    ui.add_space(2.0);
                    ui.add(
                        egui::TextEdit::singleline(text)
                            .password(true)
                            .hint_text(hint)
                            .frame(egui::Frame::NONE)
                            .desired_width(ui.available_width()),
                    );
                });
            });
    }

    /// Renders the styled multiline text input container
    pub fn render_styled_multiline_input(ui: &mut egui::Ui, text: &mut String, hint: &str, rows: usize) {
        egui::Frame::new()
            .fill(Color32::from_rgb(248, 250, 252))
            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
            .corner_radius(6.0)
            .inner_margin(egui::Margin::symmetric(10, 7))
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(text)
                        .hint_text(hint)
                        .desired_rows(rows)
                        .frame(egui::Frame::NONE)
                        .desired_width(ui.available_width()),
                );
            });
    }

    /// Renders a telemetry pill badge
    pub fn render_telemetry_pill(ui: &mut egui::Ui, icon: &str, label: &str, bg: Color32, fg: Color32) {
        egui::Frame::new()
            .fill(bg)
            .corner_radius(12.0)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(3.0, 0.0);
                    if !icon.is_empty() {
                        ui.label(egui::RichText::new(icon).size(10.5));
                    }
                    ui.label(egui::RichText::new(label).strong().size(10.5).color(fg));
                });
            });
    }

    /// Returns the descriptive visual icon for any modular node type
    pub fn get_node_icon(node_type: &NodeType) -> &'static str {
        match node_type {
            NodeType::Source => "🎵",
            NodeType::Endpoint => "🎧",
            NodeType::Dsp | NodeType::SavedEqPreset(_) => "🎛️",
            NodeType::TubeWarmer => "🔥",
            NodeType::StereoWidener => "↔️",
            NodeType::VolumeBooster => "🔊",
            NodeType::VolumeAttenuator => "🔉",
            NodeType::SteppedAttenuator => "🪜",
            NodeType::BassGuard => "🛡️",
            NodeType::DynamicLoudness => "🌙",
            NodeType::AnalogPreamp => "📻",
            NodeType::OpticalLeveler => "🎚️",
            NodeType::LowPassFilter => "🌊",
            NodeType::HighPassFilter => "🏔️",
            NodeType::BandPassFilter => "🎯",
            NodeType::Splitter => "🔀",
            NodeType::Mixer => "🎛️",
            NodeType::Crossover => "✂️",
            NodeType::StereoBalance => "⚖️",
            _ => "⚡",
        }
    }

    /// Formats a concise parameter summary for display inside a mini-canvas node
    pub fn format_preset_node_summary(node: &PresetNode) -> String {
        if !node.sub1.is_empty() {
            if node.sub1.len() > 18 {
                format!("{}...", &node.sub1[..16])
            } else {
                node.sub1.clone()
            }
        } else {
            match &node.node_type {
                NodeType::Source => "RAM Buffer In".to_string(),
                NodeType::Endpoint => "DAC Output".to_string(),
                NodeType::TubeWarmer => format!("Drive: {:.0}%", node.param_val),
                NodeType::StereoWidener => format!("Width: {:.0}%", node.param_val),
                NodeType::VolumeBooster => format!("Gain: +{:.1}dB", node.param_val),
                NodeType::VolumeAttenuator => format!("Atten: -{:.1}dB", node.param_val),
                NodeType::SteppedAttenuator => format!("Step: {:.0}dB", node.param_val),
                NodeType::BassGuard => format!("Thresh: {:.0}Hz", node.param_val),
                NodeType::DynamicLoudness => format!("Loud: {:.0}%", node.param_val),
                NodeType::AnalogPreamp => format!("Drive: {:.0}%", node.param_val),
                NodeType::OpticalLeveler => format!("Red: {:.0}%", node.param_val),
                NodeType::LowPassFilter => format!("Cutoff: {:.0}Hz", node.param_val),
                NodeType::HighPassFilter => format!("Cutoff: {:.0}Hz", node.param_val),
                NodeType::BandPassFilter => format!("Center: {:.0}Hz", node.param_val),
                NodeType::Splitter => "1 In ➔ 2 Out".to_string(),
                NodeType::Mixer => format!("Mix: {:.0}%", node.param_val),
                NodeType::Crossover => format!("XOver: {:.0}Hz", node.param_val),
                NodeType::StereoBalance => format!("Bal: {:.0}%", node.param_val),
                NodeType::Dsp => "10-Band EQ".to_string(),
                NodeType::SavedEqPreset(name) => format!("EQ: {}", name),
                _ => "Active".to_string(),
            }
        }
    }

    /// Renders an authentic, zoom-to-fit mini modular studio canvas preview
    /// with exact dot grid, modular node cards, accent bars, I/O jacks, and cubic bezier patchbay cables.
    pub fn render_mini_canvas_preview(ui: &mut egui::Ui, preset: &SignalChainPreset, size: Vec2) {
        let (canvas_rect, _resp) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter().with_clip_rect(canvas_rect);

        // Canvas Background: Clean Nordic Studio Frame
        painter.rect_filled(canvas_rect, 8.0, Color32::from_rgb(248, 250, 252));
        painter.rect_stroke(canvas_rect, 8.0, Stroke::new(1.2, Color32::from_rgb(226, 232, 240)), egui::StrokeKind::Inside);

        if preset.nodes.is_empty() {
            painter.text(
                canvas_rect.center(),
                egui::Align2::CENTER_CENTER,
                "⚡ Kanvas Studio Kosong",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(148, 163, 184),
            );
            return;
        }

        // Calculate Bounding Box of all nodes in canvas space
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node in &preset.nodes {
            let sz = crate::ui::patchbay::types::get_node_size(&node.node_type);
            min_x = min_x.min(node.pos[0]);
            min_y = min_y.min(node.pos[1]);
            max_x = max_x.max(node.pos[0] + sz.x);
            max_y = max_y.max(node.pos[1] + sz.y);
        }

        let graph_w = (max_x - min_x).max(200.0);
        let graph_h = (max_y - min_y).max(90.0);

        let pad_x = 16.0;
        let pad_y = 14.0;
        let avail_w = (canvas_rect.width() - pad_x * 2.0).max(40.0);
        let avail_h = (canvas_rect.height() - pad_y * 2.0).max(40.0);

        let scale = (avail_w / graph_w).min(avail_h / graph_h).clamp(0.24, 0.85);

        let scaled_w = graph_w * scale;
        let scaled_h = graph_h * scale;
        let origin_x = canvas_rect.min.x + (canvas_rect.width() - scaled_w) * 0.5 - min_x * scale;
        let origin_y = canvas_rect.min.y + (canvas_rect.height() - scaled_h) * 0.5 - min_y * scale;

        let transform = |x: f32, y: f32| -> Pos2 {
            Pos2::new(origin_x + x * scale, origin_y + y * scale)
        };

        // Draw authentic dot grid
        let grid_step = (24.0 * scale).max(12.0);
        let mut gx = canvas_rect.min.x + 8.0;
        while gx < canvas_rect.max.x - 8.0 {
            let mut gy = canvas_rect.min.y + 8.0;
            while gy < canvas_rect.max.y - 8.0 {
                painter.circle_filled(Pos2::new(gx, gy), 1.0, Color32::from_rgb(226, 232, 240));
                gy += grid_step;
            }
            gx += grid_step;
        }

        // 1. Draw Cubic Bezier Patchbay Cables
        for conn in &preset.connections {
            let from_node = preset.nodes.iter().find(|n| n.id == conn.from_node);
            let to_node = preset.nodes.iter().find(|n| n.id == conn.to_node);

            if let (Some(from_n), Some(to_n)) = (from_node, to_node) {
                let from_sz = crate::ui::patchbay::types::get_node_size(&from_n.node_type);
                let to_sz = crate::ui::patchbay::types::get_node_size(&to_n.node_type);

                let pin_out_y = if from_n.node_type == NodeType::Splitter || from_n.node_type == NodeType::Crossover {
                    if conn.from_port == 1 { 12.0 } else { -12.0 }
                } else {
                    0.0
                };
                let p_start_world = Pos2::new(from_n.pos[0] + from_sz.x, from_n.pos[1] + from_sz.y * 0.5 + pin_out_y);

                let pin_in_y = if to_n.node_type == NodeType::Mixer {
                    if conn.to_port == 1 { 12.0 } else { -12.0 }
                } else {
                    0.0
                };
                let p_end_world = Pos2::new(to_n.pos[0], to_n.pos[1] + to_sz.y * 0.5 + pin_in_y);

                let p_start = transform(p_start_world.x, p_start_world.y);
                let p_end = transform(p_end_world.x, p_end_world.y);

                let dx = (p_end.x - p_start.x).abs().max(25.0 * scale) * 0.5;
                let cp1 = Pos2::new(p_start.x + dx, p_start.y);
                let cp2 = Pos2::new(p_end.x - dx, p_end.y);

                let steps = 18;
                let mut pts = Vec::with_capacity(steps + 1);
                for s in 0..=steps {
                    let t = s as f32 / steps as f32;
                    let u = 1.0 - t;
                    let x = u * u * u * p_start.x + 3.0 * u * u * t * cp1.x + 3.0 * u * t * t * cp2.x + t * t * t * p_end.x;
                    let y = u * u * u * p_start.y + 3.0 * u * u * t * cp1.y + 3.0 * u * t * t * cp2.y + t * t * t * p_end.y;
                    pts.push(Pos2::new(x, y));
                }

                // Glow underlay + main cable line
                painter.add(egui::Shape::line(pts.clone(), Stroke::new((3.2 * scale).clamp(1.5, 3.2), Color32::from_rgb(199, 210, 254))));
                painter.add(egui::Shape::line(pts, Stroke::new((1.8 * scale).clamp(1.0, 2.0), Color32::from_rgb(79, 70, 229))));

                // Jack plugs
                let jack_r = (2.6 * scale).clamp(1.5, 3.0);
                painter.circle_filled(p_start, jack_r, Color32::from_rgb(67, 56, 202));
                painter.circle_filled(p_end, jack_r, Color32::from_rgb(67, 56, 202));
            }
        }

        // 2. Draw Modular Nodes
        for node in &preset.nodes {
            let sz = crate::ui::patchbay::types::get_node_size(&node.node_type);
            let top_left = transform(node.pos[0], node.pos[1]);
            let node_w = sz.x * scale;
            let node_h = sz.y * scale;
            let node_rect = Rect::from_min_size(top_left, Vec2::new(node_w, node_h));

            // Node card background
            painter.rect_filled(node_rect, (6.0 * scale).clamp(3.0, 6.0), Color32::WHITE);
            painter.rect_stroke(
                node_rect,
                (6.0 * scale).clamp(3.0, 6.0),
                Stroke::new((1.2 * scale).clamp(0.8, 1.4), Color32::from_rgb(203, 213, 225)),
                egui::StrokeKind::Inside,
            );

            // Header accent bar
            let strip_h = (17.0 * scale).clamp(9.0, 18.0);
            let strip_rect = Rect::from_min_size(node_rect.min, Vec2::new(node_rect.width(), strip_h));
            let strip_bg = match &node.node_type {
                NodeType::Source => Color32::from_rgb(238, 242, 255),
                NodeType::Endpoint => Color32::from_rgb(240, 253, 244),
                NodeType::Dsp | NodeType::SavedEqPreset(_) => Color32::from_rgb(250, 245, 255),
                NodeType::TubeWarmer | NodeType::AnalogPreamp => Color32::from_rgb(254, 243, 199),
                _ => Color32::from_rgb(241, 245, 249),
            };
            painter.rect_filled(strip_rect, (5.0 * scale).clamp(2.5, 5.0), strip_bg);

            // Header Text: Icon + Title
            let font_sz = (10.0 * scale).clamp(7.5, 12.0);
            let icon_str = Self::get_node_icon(&node.node_type);
            let title_text = format!("{} {}", icon_str, node.title);
            let max_chars = if scale < 0.45 { 13 } else { 20 };
            let display_title = if title_text.chars().count() > max_chars {
                format!("{}...", title_text.chars().take(max_chars - 2).collect::<String>())
            } else {
                title_text
            };
            painter.text(
                Pos2::new(node_rect.min.x + 4.5 * scale.max(0.6), strip_rect.center().y),
                egui::Align2::LEFT_CENTER,
                display_title,
                egui::FontId::proportional(font_sz),
                Color32::from_rgb(15, 23, 42),
            );

            // Subtext / Parameter summary
            let sub_font_sz = (8.5 * scale).clamp(6.5, 10.0);
            let sub_y = node_rect.min.y + strip_h + (node_rect.height() - strip_h) * 0.45;
            let param_text = Self::format_preset_node_summary(node);
            painter.text(
                Pos2::new(node_rect.min.x + 5.5 * scale.max(0.6), sub_y),
                egui::Align2::LEFT_CENTER,
                param_text,
                egui::FontId::proportional(sub_font_sz),
                Color32::from_rgb(100, 116, 139),
            );

            // Port Jack indicators
            let port_r = (3.2 * scale).clamp(1.8, 3.5);
            if node.node_type != NodeType::Source {
                let in_jack = Pos2::new(node_rect.min.x, node_rect.center().y);
                painter.circle_filled(in_jack, port_r, Color32::from_rgb(148, 163, 184));
                painter.circle_stroke(in_jack, port_r, Stroke::new(1.0, Color32::WHITE));
            }
            if node.node_type != NodeType::Endpoint {
                let out_jack = Pos2::new(node_rect.max.x, node_rect.center().y);
                painter.circle_filled(out_jack, port_r, Color32::from_rgb(79, 70, 229));
                painter.circle_stroke(out_jack, port_r, Stroke::new(1.0, Color32::WHITE));
            }
        }

        // Telemetry watermark bottom-left
        let mode_str = if preset.chain_mode == ChainMode::DspActive { "⚡ DSP Active (64-bit)" } else { "🛡️ Bit-Perfect Direct" };
        let watermark = format!("{} • {} Modul • {} Kabel", mode_str, preset.nodes.len(), preset.connections.len());
        painter.text(
            Pos2::new(canvas_rect.min.x + 8.0, canvas_rect.max.y - 7.0),
            egui::Align2::LEFT_BOTTOM,
            watermark,
            egui::FontId::proportional(9.0),
            Color32::from_rgb(148, 163, 184),
        );
    }

    /// Renders the rich interactive inspection hover tooltip for a preset card
    #[allow(dead_code)]
    pub fn render_preset_hover_tooltip(ui: &mut egui::Ui, entry: &LocalPresetEntry) {
        egui::Frame::new()
            .fill(Color32::from_rgb(15, 23, 42))
            .stroke(Stroke::new(1.2, Color32::from_rgb(99, 102, 241)))
            .corner_radius(10.0)
            .inner_margin(egui::Margin::symmetric(14, 12))
            .show(ui, |ui| {
                ui.set_max_width(360.0);
                ui.horizontal(|ui| {
                    let (badge_rect, _) = ui.allocate_exact_size(Vec2::splat(26.0), egui::Sense::hover());
                    ui.painter().rect_filled(badge_rect, 6.0, Color32::from_rgb(30, 41, 59));
                    ui.painter().text(
                        badge_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🔍",
                        egui::FontId::proportional(13.0),
                        Color32::WHITE,
                    );
                    ui.add_space(4.0);
                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!("PREVIEW: {}", entry.metadata.name))
                                .strong()
                                .size(12.0)
                                .color(Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new(format!("Oleh {} • Lunula Studio", entry.metadata.author))
                                .size(9.5)
                                .color(Color32::from_rgb(148, 163, 184)),
                        );
                    });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                if let Some(ref preset) = entry.cached_preset {
                    ui.label(egui::RichText::new("⚡ PRATINJAU KANVAS STUDIO:").strong().size(9.5).color(Color32::from_rgb(129, 140, 248)));
                    ui.add_space(3.0);
                    Self::render_mini_canvas_preview(ui, preset, Vec2::new(330.0, 160.0));
                    ui.add_space(4.0);
                } else if entry.metadata.has_password {
                    ui.label(egui::RichText::new("🔐 PRESET TERPROTEKSI SANDI").strong().size(10.5).color(Color32::from_rgb(251, 191, 36)));
                    ui.label(egui::RichText::new("Rantai sinyal terenkripsi ChaCha20. Klik 'Terapkan' dan ketik kata sandi untuk membuka.").size(9.5).color(Color32::from_rgb(203, 213, 225)));
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🛡️ Integritas HMAC-SHA256").size(9.0).color(Color32::from_rgb(16, 185, 129)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("Klik kartu untuk inspeksi").size(9.0).color(Color32::from_rgb(148, 163, 184)));
                    });
                });
            });
    }

    /// Renders the modal dialog for creating, browsing, and managing signal chain presets.
    pub fn render_export_preset_modal(&mut self, ctx: &egui::Context) {
        if !self.show_export_preset_modal {
            return;
        }

        let mut close_requested = false;
        let mut do_save_to_collection = false;
        let mut do_export_to_file = false;
        let mut do_import_to_collection = false;
        let mut action_apply_local: Option<(PathBuf, bool)> = None;
        let mut action_delete_local: Option<PathBuf> = None;
        let mut action_export_local: Option<PathBuf> = None;
        let mut action_duplicate_local: Option<PathBuf> = None;

        let screen_rect = ctx.input(|i| i.raw.screen_rect).unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));
        #[allow(deprecated)]
        let modal_area_id = egui::Id::new("export_preset_modal_area");

        egui::Area::new(modal_area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                // Dimmed Frosted Backdrop
                ui.painter().rect_filled(screen_rect, 0.0, Color32::from_black_alpha(170));

                let backdrop_click = ui.interact(screen_rect, modal_area_id.with("backdrop_click"), egui::Sense::click());
                if backdrop_click.clicked() {
                    close_requested = true;
                }

                let modal_w = 780.0f32.min(screen_rect.width() - 32.0);
                let modal_h = 620.0f32.min(screen_rect.height() - 32.0);
                let modal_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

                let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(modal_rect));

                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.6, Color32::from_rgb(79, 70, 229))) // Royal Indigo Border
                    .corner_radius(16.0)
                    .inner_margin(22.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 20],
                        blur: 48,
                        spread: 0,
                        color: Color32::from_black_alpha(160),
                    })
                    .show(&mut child_ui, |ui| {
                        // --- HEADER BAR ---
                        ui.horizontal(|ui| {
                            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                            ui.painter().rect_filled(icon_rect, 8.0, Color32::from_rgb(238, 242, 255));
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "💾",
                                egui::FontId::proportional(18.0),
                                Color32::WHITE,
                            );

                            ui.add_space(8.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("MANAJER PRESET STUDIO (.LNL)")
                                        .strong()
                                        .size(14.5)
                                        .color(Color32::from_rgb(15, 23, 42)),
                                );
                                ui.label(
                                    egui::RichText::new("Enkripsi ChaCha20-Poly1305 / HMAC-SHA256 • Integritas Signal Chain")
                                        .size(11.0)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("✕").size(14.0).strong().color(Color32::from_rgb(100, 116, 139)))
                                        .fill(Color32::from_rgb(241, 245, 249))
                                        .corner_radius(14.0)
                                        .min_size(Vec2::splat(28.0)),
                                )
                                .on_hover_text("Tutup jendela")
                                .clicked()
                                {
                                    close_requested = true;
                                }
                            });
                        });

                        ui.add_space(10.0);

                        // --- TAB SWITCHER BAR ---
                        let local_presets = list_local_presets();
                        let preset_count = local_presets.len();
                        let binding_count = self.preset_bindings.len();

                        ui.horizontal(|ui| {
                            let is_tab0 = self.preset_modal_tab == 0;
                            let is_tab1 = self.preset_modal_tab == 1;
                            let is_tab2 = self.preset_modal_tab == 2;

                            let btn_save = egui::Button::new(
                                egui::RichText::new("💾 Simpan Baru").strong().size(11.0)
                                    .color(if is_tab0 { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) })
                            )
                            .fill(if is_tab0 { Color32::from_rgb(238, 242, 255) } else { Color32::from_rgb(248, 250, 252) })
                            .stroke(Stroke::new(1.0, if is_tab0 { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                            .corner_radius(6.0);

                            let btn_manage = egui::Button::new(
                                egui::RichText::new(format!("📚 Koleksi ({})", preset_count)).strong().size(11.0)
                                    .color(if is_tab1 { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) })
                            )
                            .fill(if is_tab1 { Color32::from_rgb(238, 242, 255) } else { Color32::from_rgb(248, 250, 252) })
                            .stroke(Stroke::new(1.0, if is_tab1 { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                            .corner_radius(6.0);

                            let btn_binding = egui::Button::new(
                                egui::RichText::new(format!("🔗 Auto-Binding ({})", binding_count)).strong().size(11.0)
                                    .color(if is_tab2 { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) })
                            )
                            .fill(if is_tab2 { Color32::from_rgb(238, 242, 255) } else { Color32::from_rgb(248, 250, 252) })
                            .stroke(Stroke::new(1.0, if is_tab2 { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                            .corner_radius(6.0);

                            if ui.add(btn_save).clicked() {
                                self.preset_modal_tab = 0;
                                self.binding_picker_open = false;
                            }
                            if ui.add(btn_manage).clicked() {
                                self.preset_modal_tab = 1;
                                self.binding_picker_open = false;
                            }
                            if ui.add(btn_binding).clicked() {
                                self.preset_modal_tab = 2;
                            }
                        });

                        ui.add_space(8.0);

                        // --- TAB CONTENT ---
                        if self.preset_modal_tab == 0 {
                            // ==========================================
                            // TAB 0: SIMPAN PRESET BARU (CREATE)
                            // ==========================================

                            // Section 1: Form Inputs
                            let col2_w = (ui.available_width() - 10.0) / 2.0;
                            ui.horizontal(|ui| {
                                ui.allocate_ui(Vec2::new(col2_w, 46.0), |ui| {
                                    ui.label(egui::RichText::new("🏷️ NAMA PRESET").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                                    ui.add_space(3.0);
                                    Self::render_styled_text_input(ui, &mut self.preset_export_name, "mis: Warm Analog Master", "");
                                });
                                ui.add_space(6.0);
                                ui.allocate_ui(Vec2::new(col2_w, 46.0), |ui| {
                                    ui.label(egui::RichText::new("👤 PEMBUAT / AUTHOR").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                                    ui.add_space(3.0);
                                    Self::render_styled_text_input(ui, &mut self.preset_export_author, "mis: Lunula Sound Lab", "");
                                });
                            });

                            ui.add_space(8.0);
                            ui.label(egui::RichText::new("📝 DESKRIPSI & KARAKTER AUDIO").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            ui.add_space(3.0);
                            Self::render_styled_multiline_input(ui, &mut self.preset_export_description, "Catatan karakter warna suara, modular routing, dan peruntukan musik...", 2);

                            ui.add_space(10.0);

                            // Section 2: Security & Protection Selector Card
                            ui.label(egui::RichText::new("🛡️ METODE ENKRIPSI & KEAMANAN").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                            ui.add_space(3.0);

                            let sec_col_w = (ui.available_width() - 8.0) / 2.0;
                            ui.horizontal(|ui| {
                                // Option A: Master Key (Open to all Lunula players)
                                let opt_a_active = !self.preset_export_use_password;
                                let opt_a_frame = egui::Frame::new()
                                    .fill(if opt_a_active { Color32::from_rgb(238, 242, 255) } else { Color32::from_rgb(248, 250, 252) })
                                    .stroke(Stroke::new(1.2, if opt_a_active { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(226, 232, 240) }))
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(10, 8));

                                opt_a_frame.show(ui, |ui| {
                                    ui.set_width(sec_col_w - 20.0);
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            if ui.radio(opt_a_active, "").clicked() {
                                                self.preset_export_use_password = false;
                                            }
                                            ui.label(egui::RichText::new("🛡️ Kunci Master Lunula").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                                        });
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Terenkripsi otomatis & langsung terbuka di Lunula Player tanpa sandi.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    });
                                });

                                ui.add_space(4.0);

                                // Option B: Custom Passphrase Protection
                                let opt_b_active = self.preset_export_use_password;
                                let opt_b_frame = egui::Frame::new()
                                    .fill(if opt_b_active { Color32::from_rgb(254, 243, 199) } else { Color32::from_rgb(248, 250, 252) })
                                    .stroke(Stroke::new(1.2, if opt_b_active { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(226, 232, 240) }))
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(10, 8));

                                opt_b_frame.show(ui, |ui| {
                                    ui.set_width(sec_col_w - 20.0);
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            if ui.radio(opt_b_active, "").clicked() {
                                                self.preset_export_use_password = true;
                                            }
                                            ui.label(egui::RichText::new("🔐 Kata Sandi Kustom").strong().size(11.5).color(Color32::from_rgb(30, 41, 59)));
                                        });
                                        ui.add_space(2.0);
                                        ui.label(egui::RichText::new("Dilindungi passphrase unik. Hanya penerima sandi yang dapat membuka.").size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                    });
                                });
                            });

                            if self.preset_export_use_password {
                                ui.add_space(6.0);
                                Self::render_styled_password_input(ui, &mut self.preset_export_password, "Masukkan kata sandi kustom pengunci preset...");
                            }

                            ui.add_space(10.0);

                            // Section 3: Telemetry Overview Card
                            egui::Frame::new()
                                .fill(Color32::from_rgb(248, 250, 252))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                .corner_radius(8.0)
                                .inner_margin(egui::Margin::symmetric(10, 7))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("⚡ STATISTIK:").strong().size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                        ui.add_space(4.0);

                                        let node_cnt = self.nodes.len();
                                        let cable_cnt = self.connections.len();
                                        let is_dsp = self.chain_mode == ChainMode::DspActive;

                                        Self::render_telemetry_pill(ui, "🎛️", &format!("{} Modul", node_cnt), Color32::from_rgb(238, 242, 255), Color32::from_rgb(79, 70, 229));
                                        Self::render_telemetry_pill(ui, "🔌", &format!("{} Kabel", cable_cnt), Color32::from_rgb(240, 253, 244), Color32::from_rgb(22, 163, 74));
                                        Self::render_telemetry_pill(
                                            ui,
                                            if is_dsp { "⚡" } else { "🛡️" },
                                            if is_dsp { "DSP Active" } else { "Bit-Perfect" },
                                            if is_dsp { Color32::from_rgb(254, 243, 199) } else { Color32::from_rgb(224, 242, 254) },
                                            if is_dsp { Color32::from_rgb(180, 83, 9) } else { Color32::from_rgb(3, 105, 161) },
                                        );
                                        Self::render_telemetry_pill(ui, "📈", "Parametric EQ", Color32::from_rgb(250, 245, 255), Color32::from_rgb(124, 58, 237));
                                    });
                                });

                            ui.add_space(14.0);

                            // Section 4: Action Footer
                            ui.horizontal(|ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("Batal").size(11.5).color(Color32::from_rgb(100, 116, 139)))
                                        .fill(Color32::from_rgb(241, 245, 249))
                                        .corner_radius(6.0)
                                        .min_size(Vec2::new(70.0, 32.0)),
                                )
                                .clicked()
                                {
                                    close_requested = true;
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Save Button 2: Export to arbitrary file
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("📦 Ekspor Berkas (.lnl)...").strong().size(11.5).color(Color32::WHITE)
                                        )
                                        .fill(Color32::from_rgb(79, 70, 229))
                                        .corner_radius(6.0)
                                        .min_size(Vec2::new(140.0, 32.0)),
                                    )
                                    .on_hover_text("Pilih lokasi penyimpanan berkas .lnl di komputer Anda")
                                    .clicked()
                                    {
                                        do_export_to_file = true;
                                    }

                                    ui.add_space(6.0);

                                    // Save Button 1: Save directly to App Library Collection
                                    if ui.add(
                                        egui::Button::new(
                                            egui::RichText::new("💾 Simpan ke Koleksi").strong().size(11.5).color(Color32::from_rgb(79, 70, 229))
                                        )
                                        .fill(Color32::from_rgb(238, 242, 255))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(165, 180, 252)))
                                        .corner_radius(6.0)
                                        .min_size(Vec2::new(125.0, 32.0)),
                                    )
                                    .on_hover_text("Simpan instan ke perpustakaan preset lokal Lunula")
                                    .clicked()
                                    {
                                        do_save_to_collection = true;
                                    }
                                });
                            });

                        } else if self.preset_modal_tab == 1 {
                            // ==========================================
                            // TAB 1: KOLEKSI PRESET (SPLIT-PANE VIEWPORT)
                            // Left Pane: Search + Cards List + Import
                            // Right Pane: Inspector Header + Real Mini-Canvas Viewport + Telemetry + Actions
                            // ==========================================
                            let filter_query = self.preset_search_query.to_lowercase();
                            let filtered_presets: Vec<LocalPresetEntry> = local_presets
                                .into_iter()
                                .filter(|p| {
                                    if filter_query.is_empty() {
                                        true
                                    } else {
                                        p.metadata.name.to_lowercase().contains(&filter_query)
                                            || p.metadata.author.to_lowercase().contains(&filter_query)
                                            || p.metadata.description.to_lowercase().contains(&filter_query)
                                    }
                                })
                                .collect();

                            let total_w = ui.available_width();
                            let left_pane_w = 260.0f32;
                            let right_pane_w = (total_w - left_pane_w - 14.0).max(340.0);
                            let content_h = modal_h - 130.0;

                            ui.horizontal(|ui| {
                                // --- LEFT COLUMN (260px): Search & Card List ---
                                ui.allocate_ui(Vec2::new(left_pane_w, content_h), |ui| {
                                    ui.vertical(|ui| {
                                        // 1. Search Bar
                                        ui.horizontal(|ui| {
                                            Self::render_styled_text_input(ui, &mut self.preset_search_query, "🔍 Cari preset...", "");
                                        });

                                        ui.add_space(6.0);

                                        // 2. Preset Cards Scroll Area
                                        let list_scroll_h = content_h - 78.0;
                                        egui::ScrollArea::vertical()
                                            .id_salt("preset_library_cards_scroll")
                                            .max_height(list_scroll_h)
                                            .auto_shrink([false, false])
                                            .show(ui, |ui| {
                                                if filtered_presets.is_empty() {
                                                    ui.vertical_centered(|ui| {
                                                        ui.add_space(36.0);
                                                        ui.label(egui::RichText::new("🔍").size(24.0));
                                                        ui.add_space(4.0);
                                                        ui.label(egui::RichText::new("Tidak ada preset yang cocok").size(11.5).color(Color32::from_rgb(100, 116, 139)));
                                                    });
                                                } else {
                                                    for (idx, entry) in filtered_presets.iter().enumerate() {
                                                        let is_selected = self.preset_selected_idx == idx;
                                                        let card_bg = if is_selected {
                                                            Color32::from_rgb(238, 242, 255)
                                                        } else {
                                                            Color32::from_rgb(248, 250, 252)
                                                        };
                                                        let border_color = if is_selected {
                                                            Color32::from_rgb(99, 102, 241)
                                                        } else {
                                                            Color32::from_rgb(226, 232, 240)
                                                        };

                                                        let (card_rect, card_resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 52.0), egui::Sense::click());
                                                        let painter = ui.painter();
                                                        painter.rect_filled(card_rect, 6.0, card_bg);
                                                        painter.rect_stroke(card_rect, 6.0, Stroke::new(if is_selected { 1.5 } else { 1.0 }, border_color), egui::StrokeKind::Inside);

                                                        // Left Icon Box
                                                        let icon_box = Rect::from_center_size(Pos2::new(card_rect.min.x + 19.0, card_rect.center().y), Vec2::splat(26.0));
                                                        let icon_bg = if is_selected { Color32::from_rgb(224, 231, 255) } else { Color32::from_rgb(241, 245, 249) };
                                                        painter.rect_filled(icon_box, 5.0, icon_bg);
                                                        let icon_str = if entry.metadata.has_password { "🔐" } else { "💾" };
                                                        painter.text(icon_box.center(), egui::Align2::CENTER_CENTER, icon_str, egui::FontId::proportional(12.5), Color32::from_rgb(79, 70, 229));

                                                        // Card Text (Title & Subtitle)
                                                        let text_left = card_rect.min.x + 38.0;
                                                        let text_w = card_rect.width() - 44.0;
                                                        let title_color = if is_selected { Color32::from_rgb(67, 56, 202) } else { Color32::from_rgb(15, 23, 42) };

                                                        let title_clip = ui.painter().with_clip_rect(Rect::from_min_size(Pos2::new(text_left, card_rect.min.y + 6.0), Vec2::new(text_w, 16.0)));
                                                        title_clip.text(Pos2::new(text_left, card_rect.min.y + 7.0), egui::Align2::LEFT_TOP, &entry.metadata.name, egui::FontId::proportional(12.0), title_color);

                                                        let sub_clip = ui.painter().with_clip_rect(Rect::from_min_size(Pos2::new(text_left, card_rect.min.y + 24.0), Vec2::new(text_w, 15.0)));
                                                        let sub_str = format!("{} • {} Modul", entry.metadata.author, entry.metadata.node_count);
                                                        sub_clip.text(Pos2::new(text_left, card_rect.min.y + 25.0), egui::Align2::LEFT_TOP, sub_str, egui::FontId::proportional(10.0), Color32::from_rgb(100, 116, 139));

                                                        if card_resp.clicked() {
                                                            self.preset_selected_idx = idx;
                                                            self.inspecting_preset = entry.cached_preset.clone();
                                                        }

                                                        let (bound_tracks, bound_albums, bound_genres) = self.get_preset_binding_targets(&entry.metadata.name);
                                                        card_resp.on_hover_ui(|ui| {
                                                            egui::Frame::new()
                                                                .fill(Color32::WHITE)
                                                                .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                                                .corner_radius(8.0)
                                                                .inner_margin(egui::Margin::symmetric(10, 8))
                                                                .show(ui, |ui| {
                                                                    ui.set_max_width(260.0);
                                                                    ui.label(egui::RichText::new(&entry.metadata.name).strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                                                    let desc = if entry.metadata.description.is_empty() { "Konfigurasi rantai audio Lunula Studio." } else { &entry.metadata.description };
                                                                    ui.label(egui::RichText::new(desc).size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                                                    ui.add_space(4.0);
                                                                    ui.separator();
                                                                    ui.add_space(4.0);

                                                                    let total = bound_tracks.len() + bound_albums.len() + bound_genres.len();
                                                                    if total == 0 {
                                                                        ui.label(egui::RichText::new("🔗 Bebas (Belum Terikat)").strong().size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                                        ui.label(egui::RichText::new("Preset ini siap dipasangkan ke lagu, album, atau genre.").size(9.5).color(Color32::from_rgb(148, 163, 184)));
                                                                    } else {
                                                                        ui.label(egui::RichText::new(format!("🔗 Terikat pada {} Target:", total)).strong().size(10.5).color(Color32::from_rgb(67, 56, 202)));
                                                                        ui.add_space(2.0);

                                                                        if !bound_tracks.is_empty() {
                                                                            let preview = if bound_tracks.len() <= 2 {
                                                                                bound_tracks.join(", ")
                                                                            } else {
                                                                                format!("{}, +{} lainnya", bound_tracks[..2].join(", "), bound_tracks.len() - 2)
                                                                            };
                                                                            ui.label(egui::RichText::new(format!("🎵 Trek: {}", preview)).size(9.5).color(Color32::from_rgb(5, 150, 105)));
                                                                        }

                                                                        if !bound_albums.is_empty() {
                                                                            let preview = if bound_albums.len() <= 2 {
                                                                                bound_albums.join(", ")
                                                                            } else {
                                                                                format!("{}, +{} lainnya", bound_albums[..2].join(", "), bound_albums.len() - 2)
                                                                            };
                                                                            ui.label(egui::RichText::new(format!("💿 Album: {}", preview)).size(9.5).color(Color32::from_rgb(79, 70, 229)));
                                                                        }

                                                                        if !bound_genres.is_empty() {
                                                                            ui.label(egui::RichText::new(format!("🎭 Genre: {}", bound_genres.join(", "))).size(9.5).color(Color32::from_rgb(124, 58, 237)));
                                                                        }
                                                                    }
                                                                });
                                                        });

                                                        ui.add_space(4.0);
                                                    }
                                                }
                                            });

                                        ui.add_space(4.0);

                                        // 3. Import Button at Bottom of Left Column
                                        if ui.add(
                                            egui::Button::new(
                                                egui::RichText::new("📂 Impor Berkas (.lnl)...").strong().size(11.0).color(Color32::from_rgb(14, 116, 144))
                                            )
                                            .fill(Color32::from_rgb(240, 253, 250))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(153, 246, 228)))
                                            .corner_radius(6.0)
                                            .min_size(Vec2::new(left_pane_w, 28.0))
                                        )
                                        .on_hover_text("Buka preset studio .lnl dari folder komputer")
                                        .clicked()
                                        {
                                            do_import_to_collection = true;
                                        }
                                    });
                                });

                                ui.add_space(6.0);
                                ui.separator();
                                ui.add_space(6.0);

                                // --- RIGHT COLUMN: Active Preset Viewport & Mini-Canvas ---
                                ui.allocate_ui(Vec2::new(right_pane_w, content_h), |ui| {
                                    if let Some(entry) = filtered_presets.get(self.preset_selected_idx).or_else(|| filtered_presets.first()) {
                                        ui.vertical(|ui| {
                                            // 1. Header of Inspected Preset
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.label(egui::RichText::new(&entry.metadata.name).strong().size(15.0).color(Color32::from_rgb(15, 23, 42)));
                                                        ui.add_space(4.0);
                                                        ui.label(egui::RichText::new(format!("oleh {}", entry.metadata.author)).size(11.0).color(Color32::from_rgb(100, 116, 139)));
                                                    });
                                                    let desc_str = if !entry.metadata.description.is_empty() {
                                                        entry.metadata.description.as_str()
                                                    } else {
                                                        "Konfigurasi rantai modul audio studio Lunula."
                                                    };
                                                    ui.label(egui::RichText::new(desc_str).size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                });
                                            });

                                            ui.add_space(6.0);

                                            // 2. Real Mini-Canvas Viewport (Width ~450px, Height ~240px)
                                            let canvas_w = right_pane_w;
                                            let canvas_h = 240.0f32;

                                            if let Some(ref preset) = entry.cached_preset {
                                                Self::render_mini_canvas_preview(ui, preset, Vec2::new(canvas_w, canvas_h));
                                            } else if entry.metadata.has_password {
                                                egui::Frame::new()
                                                    .fill(Color32::from_rgb(254, 243, 199))
                                                    .stroke(Stroke::new(1.0, Color32::from_rgb(251, 191, 36)))
                                                    .corner_radius(8.0)
                                                    .inner_margin(egui::Margin::symmetric(16, 36))
                                                    .show(ui, |ui| {
                                                        ui.set_width(canvas_w - 32.0);
                                                        ui.set_height(canvas_h - 72.0);
                                                        ui.vertical_centered(|ui| {
                                                            ui.label(egui::RichText::new("🔐 PRESET TERKUNCI KATA SANDI").strong().size(13.0).color(Color32::from_rgb(180, 83, 9)));
                                                            ui.add_space(4.0);
                                                            ui.label(egui::RichText::new("Preset ini dienkripsi dengan ChaCha20-Poly1305.\nKlik tombol '⚡ Terapkan' di bawah dan masukkan sandi untuk membuka kanvas.").size(10.5).color(Color32::from_rgb(146, 64, 14)));
                                                        });
                                                    });
                                            } else {
                                                egui::Frame::new()
                                                    .fill(Color32::from_rgb(248, 250, 252))
                                                    .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                                    .corner_radius(8.0)
                                                    .inner_margin(egui::Margin::symmetric(16, 36))
                                                    .show(ui, |ui| {
                                                        ui.set_width(canvas_w - 32.0);
                                                        ui.set_height(canvas_h - 72.0);
                                                        ui.vertical_centered(|ui| {
                                                            ui.label(egui::RichText::new("🎛️ Data kanvas preset sedang disiapkan...").size(12.0).color(Color32::from_rgb(148, 163, 184)));
                                                        });
                                                    });
                                            }

                                            ui.add_space(6.0);

                                            // 3. Telemetry Pills
                                            ui.horizontal(|ui| {
                                                Self::render_telemetry_pill(
                                                    ui,
                                                    "📦",
                                                    &format!("{} Modul Audio", entry.metadata.node_count),
                                                    Color32::from_rgb(238, 242, 255),
                                                    Color32::from_rgb(79, 70, 229),
                                                );
                                                Self::render_telemetry_pill(
                                                    ui,
                                                    "🔌",
                                                    &format!("{} Kabel Sinyal", entry.metadata.connection_count),
                                                    Color32::from_rgb(240, 253, 244),
                                                    Color32::from_rgb(22, 163, 74),
                                                );
                                                if entry.metadata.has_password {
                                                    Self::render_telemetry_pill(
                                                        ui,
                                                        "🔐",
                                                        "ChaCha20 Passphrase",
                                                        Color32::from_rgb(254, 243, 199),
                                                        Color32::from_rgb(180, 83, 9),
                                                    );
                                                } else {
                                                    Self::render_telemetry_pill(
                                                        ui,
                                                        "🛡️",
                                                        "Master Key Valid",
                                                        Color32::from_rgb(240, 253, 250),
                                                        Color32::from_rgb(13, 148, 136),
                                                    );
                                                }
                                            });

                                            ui.add_space(8.0);

                                            // 4. Action Buttons (Apply, Duplicate, Export, Delete)
                                            ui.horizontal(|ui| {
                                                let apply_btn = egui::Button::new(
                                                    egui::RichText::new("⚡ Terapkan").strong().size(11.5).color(Color32::WHITE)
                                                )
                                                .fill(Color32::from_rgb(79, 70, 229))
                                                .corner_radius(6.0)
                                                .min_size(Vec2::new(130.0, 32.0));

                                                if ui.add(apply_btn).on_hover_text("Terapkan seluruh rantai sinyal ini ke studio aktif").clicked() {
                                                    action_apply_local = Some((entry.file_path.clone(), entry.metadata.has_password));
                                                }

                                                ui.add_space(4.0);

                                                let dup_btn = egui::Button::new(
                                                    egui::RichText::new("📋 Duplikat").strong().size(11.0).color(Color32::from_rgb(79, 70, 229))
                                                )
                                                .fill(Color32::from_rgb(238, 242, 255))
                                                .stroke(Stroke::new(1.0, Color32::from_rgb(199, 210, 254)))
                                                .corner_radius(6.0)
                                                .min_size(Vec2::new(88.0, 32.0));

                                                if ui.add(dup_btn).on_hover_text("Duplikasi preset ini untuk membuat variasi baru tanpa mengubah aslinya").clicked() {
                                                    action_duplicate_local = Some(entry.file_path.clone());
                                                }

                                                ui.add_space(4.0);

                                                if ui.add(
                                                    egui::Button::new(egui::RichText::new("📤 Ekspor").size(11.0).color(Color32::from_rgb(71, 85, 105)))
                                                        .fill(Color32::from_rgb(248, 250, 252))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                                        .corner_radius(6.0)
                                                        .min_size(Vec2::new(76.0, 32.0))
                                                )
                                                .on_hover_text("Ekspor salinan berkas preset .lnl ini")
                                                .clicked()
                                                {
                                                    action_export_local = Some(entry.file_path.clone());
                                                }

                                                ui.add_space(4.0);

                                                if ui.add(
                                                    egui::Button::new(egui::RichText::new("🗑️").size(11.0).color(Color32::from_rgb(239, 68, 68)))
                                                        .fill(Color32::WHITE)
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(254, 202, 202)))
                                                        .corner_radius(6.0)
                                                        .min_size(Vec2::new(36.0, 32.0))
                                                )
                                                .on_hover_text("Hapus preset dari koleksi lokal")
                                                .clicked()
                                                {
                                                    action_delete_local = Some(entry.file_path.clone());
                                                }
                                            });
                                        });
                                    } else {
                                        ui.vertical_centered(|ui| {
                                            ui.add_space(60.0);
                                            ui.label(egui::RichText::new("👈 Pilih preset di panel kiri untuk melihat pratinjau kanvas").size(12.5).color(Color32::from_rgb(148, 163, 184)));
                                        });
                                    }
                                });
                            });

                        } else {
                            // ==========================================
                            // TAB 2: AUTO-BINDING (TRACK / ALBUM / GENRE)
                            // ==========================================
                            let preset_names: Vec<String> = local_presets.iter().map(|p| p.metadata.name.clone()).collect();
                            if !preset_names.is_empty() && !preset_names.contains(&self.binding_selected_preset) {
                                self.binding_selected_preset = preset_names[0].clone();
                            }

                            if self.binding_picker_open {
                                // ----------------------------------------------------
                                // SUB-VIEW: INTERACTIVE VISUAL LIBRARY PICKER
                                // ----------------------------------------------------
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(248, 250, 252))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(12, 10))
                                    .show(ui, |ui| {
                                        // Header row: Back button + Mode Switcher
                                        ui.horizontal(|ui| {
                                            let back_btn = egui::Button::new(
                                                egui::RichText::new("⬅ Kembali ke Aturan").strong().size(11.0).color(Color32::from_rgb(79, 70, 229))
                                            )
                                            .fill(Color32::from_rgb(238, 242, 255))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(199, 210, 254)))
                                            .corner_radius(6.0);

                                            if ui.add(back_btn).clicked() {
                                                self.binding_picker_open = false;
                                                self.binding_picker_search.clear();
                                                self.binding_picker_selected_key.clear();
                                                self.binding_picker_selected_label.clear();
                                            }

                                            ui.add_space(8.0);
                                            ui.label(egui::RichText::new("PILIH TARGET DARI KOLEKSI AUDIO:").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));

                                            let modes = [("🎵 Trek Lagu", 0), ("💿 Album", 1), ("🎭 Genre", 2)];
                                            for (lbl, mode_val) in modes {
                                                let is_active = self.binding_picker_mode == mode_val;
                                                let btn = egui::Button::new(
                                                    egui::RichText::new(lbl).strong().size(10.5)
                                                        .color(if is_active { Color32::from_rgb(67, 56, 202) } else { Color32::from_rgb(100, 116, 139) })
                                                )
                                                .fill(if is_active { Color32::from_rgb(238, 242, 255) } else { Color32::WHITE })
                                                .stroke(Stroke::new(1.0, if is_active { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                                                .corner_radius(6.0);

                                                if ui.add(btn).clicked() {
                                                    self.binding_picker_mode = mode_val;
                                                    self.binding_picker_search.clear();
                                                    self.binding_picker_selected_key.clear();
                                                    self.binding_picker_selected_label.clear();
                                                }
                                            }
                                        });

                                        ui.add_space(8.0);

                                        // Search Bar
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("🔍").size(12.0));
                                            let hint = match self.binding_picker_mode {
                                                0 => "Ketik judul trek lagu atau nama artis untuk menyaring...",
                                                1 => "Ketik nama album atau artis...",
                                                _ => "Ketik nama genre musik (mis: Rock, Jazz, Pop)...",
                                            };
                                            ui.add(
                                                egui::TextEdit::singleline(&mut self.binding_picker_search)
                                                    .hint_text(hint)
                                                    .desired_width(ui.available_width() - 85.0)
                                            );
                                            if !self.binding_picker_search.is_empty() {
                                                if ui.add(
                                                    egui::Button::new(egui::RichText::new("✕ Hapus").size(10.0).color(Color32::from_rgb(100, 116, 139)))
                                                        .fill(Color32::WHITE)
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                                        .corner_radius(4.0)
                                                ).clicked() {
                                                    self.binding_picker_search.clear();
                                                }
                                            }
                                        });

                                        ui.add_space(8.0);

                                        // Pre-calculate data from self.playlist to avoid borrow conflicts
                                        let search = self.binding_picker_search.trim().to_lowercase();
                                        let selected_key = self.binding_picker_selected_key.clone();

                                        // Render List based on binding_picker_mode
                                        let picker_list_h = (ui.available_height() - 110.0).max(180.0);
                                        egui::ScrollArea::vertical()
                                            .id_salt("binding_picker_items_scroll")
                                            .max_height(picker_list_h)
                                            .auto_shrink([false, false])
                                            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                                            .show(ui, |ui| {
                                                match self.binding_picker_mode {
                                                    0 => {
                                                        // --- MODE 0: TREK LAGU ---
                                                        let filtered_tracks: Vec<(String, String, String, f32, f32)> = self.playlist
                                                            .iter()
                                                            .filter(|t| {
                                                                if search.is_empty() {
                                                                    true
                                                                } else {
                                                                    t.title.to_lowercase().contains(&search)
                                                                        || t.artist.to_lowercase().contains(&search)
                                                                        || t.album.to_lowercase().contains(&search)
                                                                }
                                                            })
                                                            .map(|t| (t.title.clone(), t.artist.clone(), t.album.clone(), t.duration, t.file_size_mb))
                                                            .collect();

                                                        if self.playlist.is_empty() {
                                                            ui.vertical_centered(|ui| {
                                                                ui.add_space(30.0);
                                                                ui.label(egui::RichText::new("📭 Library masih kosong. Tambahkan lagu atau folder ke playlist terlebih dahulu.").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                                            });
                                                        } else if filtered_tracks.is_empty() {
                                                            ui.vertical_centered(|ui| {
                                                                ui.add_space(30.0);
                                                                ui.label(egui::RichText::new("🔍 Tidak ada lagu yang cocok dengan pencarian.").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                                            });
                                                        } else {
                                                            let mut chosen_track: Option<(String, String)> = None;
                                                            for (title, artist, album, dur, size_mb) in filtered_tracks {
                                                                let is_selected = selected_key.eq_ignore_ascii_case(&title);
                                                                let existing_bind_opt = self.preset_bindings.iter()
                                                                    .find(|b| b.target_type == crate::preset::PresetBindingType::Track && b.target_key.eq_ignore_ascii_case(&title))
                                                                    .map(|b| b.preset_name.clone());

                                                                let frame = egui::Frame::new()
                                                                    .fill(if is_selected { Color32::from_rgb(238, 242, 255) } else { Color32::WHITE })
                                                                    .stroke(Stroke::new(if is_selected { 1.5 } else { 1.0 }, if is_selected { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(226, 232, 240) }))
                                                                    .corner_radius(6.0)
                                                                    .inner_margin(egui::Margin::symmetric(10, 6));

                                                                let resp = frame.show(ui, |ui| {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(egui::RichText::new("🎵").size(14.0));
                                                                        ui.vertical(|ui| {
                                                                            ui.horizontal(|ui| {
                                                                                ui.label(egui::RichText::new(&title).strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                                                                if let Some(ref eb_preset) = existing_bind_opt {
                                                                                    Self::render_telemetry_pill(ui, "⚠️", &format!("Terikat: {}", eb_preset), Color32::from_rgb(254, 243, 199), Color32::from_rgb(180, 83, 9));
                                                                                }
                                                                            });
                                                                            let mins = (dur as u32) / 60;
                                                                            let secs = (dur as u32) % 60;
                                                                            let alb_str = if album.is_empty() { "Single".to_string() } else { album.clone() };
                                                                            ui.label(egui::RichText::new(format!("{} • {} • {:.1} MB • {:02}:{:02}", artist, alb_str, size_mb, mins, secs)).size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                                                        });

                                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                            let btn = egui::Button::new(
                                                                                egui::RichText::new(if is_selected { "✓ Terpilih" } else { "Pilih" }).size(10.0).strong()
                                                                                    .color(if is_selected { Color32::WHITE } else { Color32::from_rgb(79, 70, 229) })
                                                                            )
                                                                            .fill(if is_selected { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(238, 242, 255) })
                                                                            .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(67, 56, 202) } else { Color32::from_rgb(199, 210, 254) }))
                                                                            .corner_radius(4.0);

                                                                            if ui.add(btn).clicked() {
                                                                                chosen_track = Some((title.clone(), format!("{} - {}", title, artist)));
                                                                            }
                                                                        });
                                                                    });
                                                                });

                                                                if resp.response.interact(egui::Sense::click()).clicked() {
                                                                    chosen_track = Some((title.clone(), format!("{} - {}", title, artist)));
                                                                }
                                                                ui.add_space(3.0);
                                                            }

                                                            if let Some((k, l)) = chosen_track {
                                                                self.binding_picker_selected_key = k;
                                                                self.binding_picker_selected_label = l;
                                                            }
                                                        }
                                                    }
                                                    1 => {
                                                        // --- MODE 1: ALBUM ---
                                                        let mut album_map: std::collections::BTreeMap<String, (String, usize)> = std::collections::BTreeMap::new();
                                                        for t in &self.playlist {
                                                            let album_name = if t.album.trim().is_empty() { "(Single / Tanpa Album)".to_string() } else { t.album.trim().to_string() };
                                                            let entry = album_map.entry(album_name).or_insert((t.artist.clone(), 0));
                                                            entry.1 += 1;
                                                        }

                                                        let filtered_albums: Vec<(String, String, usize)> = album_map
                                                            .into_iter()
                                                            .filter(|(album, (artist, _))| {
                                                                if search.is_empty() {
                                                                    true
                                                                } else {
                                                                    album.to_lowercase().contains(&search) || artist.to_lowercase().contains(&search)
                                                                }
                                                            })
                                                            .map(|(album, (artist, count))| (album, artist, count))
                                                            .collect();

                                                        if filtered_albums.is_empty() {
                                                            ui.vertical_centered(|ui| {
                                                                ui.add_space(30.0);
                                                                ui.label(egui::RichText::new("🔍 Tidak ada album yang cocok dengan pencarian.").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                                            });
                                                        } else {
                                                            let mut chosen_album: Option<(String, String)> = None;
                                                            for (album, artist, count) in filtered_albums {
                                                                let is_selected = selected_key.eq_ignore_ascii_case(&album);
                                                                let existing_bind_opt = self.preset_bindings.iter()
                                                                    .find(|b| b.target_type == crate::preset::PresetBindingType::Album && b.target_key.eq_ignore_ascii_case(&album))
                                                                    .map(|b| b.preset_name.clone());

                                                                let frame = egui::Frame::new()
                                                                    .fill(if is_selected { Color32::from_rgb(238, 242, 255) } else { Color32::WHITE })
                                                                    .stroke(Stroke::new(if is_selected { 1.5 } else { 1.0 }, if is_selected { Color32::from_rgb(99, 102, 241) } else { Color32::from_rgb(226, 232, 240) }))
                                                                    .corner_radius(6.0)
                                                                    .inner_margin(egui::Margin::symmetric(10, 6));

                                                                let resp = frame.show(ui, |ui| {
                                                                    ui.horizontal(|ui| {
                                                                        ui.label(egui::RichText::new("💿").size(14.0));
                                                                        ui.vertical(|ui| {
                                                                            ui.horizontal(|ui| {
                                                                                ui.label(egui::RichText::new(&album).strong().size(11.5).color(Color32::from_rgb(15, 23, 42)));
                                                                                if let Some(ref eb_preset) = existing_bind_opt {
                                                                                    Self::render_telemetry_pill(ui, "⚠️", &format!("Terikat: {}", eb_preset), Color32::from_rgb(254, 243, 199), Color32::from_rgb(180, 83, 9));
                                                                                }
                                                                            });
                                                                            ui.label(egui::RichText::new(format!("Artis: {} • {} Trek di Library", artist, count)).size(10.0).color(Color32::from_rgb(100, 116, 139)));
                                                                        });

                                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                            let btn = egui::Button::new(
                                                                                egui::RichText::new(if is_selected { "✓ Terpilih" } else { "Pilih" }).size(10.0).strong()
                                                                                    .color(if is_selected { Color32::WHITE } else { Color32::from_rgb(79, 70, 229) })
                                                                            )
                                                                            .fill(if is_selected { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(238, 242, 255) })
                                                                            .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(67, 56, 202) } else { Color32::from_rgb(199, 210, 254) }))
                                                                            .corner_radius(4.0);

                                                                            if ui.add(btn).clicked() {
                                                                                chosen_album = Some((album.clone(), format!("Album: {}", album)));
                                                                            }
                                                                        });
                                                                    });
                                                                });

                                                                if resp.response.interact(egui::Sense::click()).clicked() {
                                                                    chosen_album = Some((album.clone(), format!("Album: {}", album)));
                                                                }
                                                                ui.add_space(3.0);
                                                            }

                                                            if let Some((k, l)) = chosen_album {
                                                                self.binding_picker_selected_key = k;
                                                                self.binding_picker_selected_label = l;
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        // --- MODE 2: GENRE ---
                                                        let mut genre_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
                                                        for t in &self.playlist {
                                                            if let Some(ref g) = t.genre {
                                                                let trimmed = g.trim();
                                                                if !trimmed.is_empty() {
                                                                    *genre_counts.entry(trimmed.to_string()).or_insert(0) += 1;
                                                                }
                                                            }
                                                        }
                                                        let standard_genres = [
                                                            "Rock", "Pop", "Jazz", "Classical", "Acoustic", "Electronic",
                                                            "Hip-Hop", "R&B", "Metal", "Ambient", "Lo-Fi", "Folk", "Indie",
                                                            "Blues", "Soul", "EDM", "Soundtrack", "Live"
                                                        ];
                                                        for sg in &standard_genres {
                                                            genre_counts.entry(sg.to_string()).or_insert(0);
                                                        }

                                                        let filtered_genres: Vec<(String, usize)> = genre_counts
                                                            .into_iter()
                                                            .filter(|(genre, _)| {
                                                                if search.is_empty() {
                                                                    true
                                                                } else {
                                                                    genre.to_lowercase().contains(&search)
                                                                }
                                                            })
                                                            .collect();

                                                        if filtered_genres.is_empty() {
                                                            ui.vertical_centered(|ui| {
                                                                ui.add_space(30.0);
                                                                ui.label(egui::RichText::new("🔍 Tidak ada genre yang cocok dengan pencarian.").size(11.0).color(Color32::from_rgb(148, 163, 184)));
                                                            });
                                                        } else {
                                                            let mut chosen_genre: Option<(String, String)> = None;
                                                            ui.horizontal_wrapped(|ui| {
                                                                for (genre, count) in filtered_genres {
                                                                    let is_selected = selected_key.eq_ignore_ascii_case(&genre);
                                                                    let count_str = if count > 0 { format!(" ({})", count) } else { String::new() };
                                                                    let btn_text = format!("🎭 {}{}", genre, count_str);
                                                                    let btn = egui::Button::new(
                                                                        egui::RichText::new(btn_text).size(10.5).strong()
                                                                            .color(if is_selected { Color32::from_rgb(124, 58, 237) } else { Color32::from_rgb(71, 85, 105) })
                                                                    )
                                                                    .fill(if is_selected { Color32::from_rgb(250, 245, 255) } else { Color32::WHITE })
                                                                    .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(168, 85, 247) } else { Color32::from_rgb(203, 213, 225) }))
                                                                    .corner_radius(6.0);

                                                                    if ui.add(btn).clicked() {
                                                                        chosen_genre = Some((genre.clone(), format!("Genre: {}", genre)));
                                                                    }
                                                                }
                                                            });

                                                            if let Some((k, l)) = chosen_genre {
                                                                self.binding_picker_selected_key = k;
                                                                self.binding_picker_selected_label = l;
                                                            }
                                                        }
                                                    }
                                                }
                                            });

                                        ui.add_space(8.0);

                                        // Bottom Confirmation & Preset Association Box
                                        egui::Frame::new()
                                            .fill(Color32::from_rgb(255, 255, 255))
                                            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                            .corner_radius(8.0)
                                            .inner_margin(egui::Margin::symmetric(12, 8))
                                            .show(ui, |ui| {
                                                let (target_type, prefix) = match self.binding_picker_mode {
                                                    0 => (crate::preset::PresetBindingType::Track, "track"),
                                                    1 => (crate::preset::PresetBindingType::Album, "album"),
                                                    _ => (crate::preset::PresetBindingType::Genre, "genre"),
                                                };
                                                let existing_bind_for_target = if self.binding_picker_selected_key.is_empty() {
                                                    None
                                                } else {
                                                    self.preset_bindings.iter().find(|b| b.target_type == target_type && b.target_key.eq_ignore_ascii_case(&self.binding_picker_selected_key))
                                                };

                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("🎯 Target:").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                                                    if self.binding_picker_selected_key.is_empty() {
                                                        ui.label(egui::RichText::new("Belum ada target dipilih (klik salah satu item di atas)").size(10.5).italics().color(Color32::from_rgb(148, 163, 184)));
                                                    } else {
                                                        let (badge, bg, fg) = match self.binding_picker_mode {
                                                            0 => ("🎵 TREK", Color32::from_rgb(236, 253, 245), Color32::from_rgb(16, 185, 129)),
                                                            1 => ("💿 ALBUM", Color32::from_rgb(238, 242, 255), Color32::from_rgb(79, 70, 229)),
                                                            _ => ("🎭 GENRE", Color32::from_rgb(250, 245, 255), Color32::from_rgb(124, 58, 237)),
                                                        };
                                                        Self::render_telemetry_pill(ui, "", badge, bg, fg);
                                                        ui.label(egui::RichText::new(&self.binding_picker_selected_label).strong().size(11.0).color(Color32::from_rgb(15, 23, 42)));
                                                    }
                                                });

                                                if let Some(eb) = existing_bind_for_target {
                                                    ui.add_space(2.0);
                                                    ui.label(
                                                        egui::RichText::new(format!("⚠️ Target ini sudah terikat ke preset '{}'. Menyimpan akan memperbarui (overwrite) presetnya.", eb.preset_name))
                                                            .size(10.0)
                                                            .color(Color32::from_rgb(217, 119, 6))
                                                    );
                                                }

                                                ui.add_space(4.0);

                                                // Preset selection chips
                                                ui.horizontal(|ui| {
                                                    ui.label(egui::RichText::new("Preset Target:").strong().size(10.5).color(Color32::from_rgb(71, 85, 105)));
                                                    ui.horizontal_wrapped(|ui| {
                                                        for p_name in &preset_names {
                                                            let is_sel = &self.binding_selected_preset == p_name;
                                                            let btn = egui::Button::new(
                                                                egui::RichText::new(p_name).size(10.0).strong()
                                                                    .color(if is_sel { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(71, 85, 105) })
                                                            )
                                                            .fill(if is_sel { Color32::from_rgb(224, 242, 254) } else { Color32::from_rgb(248, 250, 252) })
                                                            .stroke(Stroke::new(1.0, if is_sel { Color32::from_rgb(14, 165, 233) } else { Color32::from_rgb(203, 213, 225) }))
                                                            .corner_radius(4.0);
                                                            if ui.add(btn).clicked() {
                                                                self.binding_selected_preset = p_name.clone();
                                                            }
                                                        }
                                                    });
                                                });

                                                ui.add_space(6.0);

                                                // Action Row
                                                ui.horizontal(|ui| {
                                                    let can_bind = !self.binding_picker_selected_key.is_empty() && !self.binding_selected_preset.is_empty();
                                                    let bind_btn = egui::Button::new(
                                                        egui::RichText::new("🔗 Simpan & Tautkan ke Preset").strong().size(11.0)
                                                            .color(if can_bind { Color32::WHITE } else { Color32::from_rgb(148, 163, 184) })
                                                    )
                                                    .fill(if can_bind { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(226, 232, 240) })
                                                    .stroke(Stroke::new(1.0, if can_bind { Color32::from_rgb(67, 56, 202) } else { Color32::from_rgb(203, 213, 225) }))
                                                    .corner_radius(6.0);

                                                    if ui.add_enabled(can_bind, bind_btn).clicked() {
                                                        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                                                        let key = self.binding_picker_selected_key.clone();
                                                        let label = self.binding_picker_selected_label.clone();
                                                        let preset_name = self.binding_selected_preset.clone();

                                                        self.preset_bindings.retain(|b| !(b.target_type == target_type && b.target_key.eq_ignore_ascii_case(&key)));
                                                        self.preset_bindings.push(crate::preset::PresetBindingRule {
                                                            id: format!("bind_{}_{}", prefix, now),
                                                            target_type,
                                                            target_key: key.clone(),
                                                            target_label: label.clone(),
                                                            preset_name: preset_name.clone(),
                                                        });
                                                        self.save_auto_bindings();
                                                        self.preset_status_notification = Some((format!("🔗 '{}' berhasil diikat ke preset '{}'!", label, preset_name), true));
                                                        self.preset_status_notification_time = Some(std::time::Instant::now());

                                                        self.binding_picker_open = false;
                                                        self.binding_picker_selected_key.clear();
                                                        self.binding_picker_selected_label.clear();
                                                    }

                                                    if ui.add(
                                                        egui::Button::new(egui::RichText::new("Batal").size(10.5).color(Color32::from_rgb(100, 116, 139)))
                                                            .fill(Color32::WHITE)
                                                            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                                            .corner_radius(6.0)
                                                    ).clicked() {
                                                        self.binding_picker_open = false;
                                                        self.binding_picker_selected_key.clear();
                                                        self.binding_picker_selected_label.clear();
                                                    }
                                                });
                                            });
                                    });
                            } else {
                                // ----------------------------------------------------
                                // MAIN TAB 2: MODERN UNIFIED AUTO-BINDING RULES MANAGER
                                // ----------------------------------------------------
                                let mut remove_binding_id: Option<String> = None;
                                let mut update_binding_preset: Option<(String, String)> = None;

                                // 1. Compact Now Playing Status Bar (~34px)
                                egui::Frame::new()
                                    .fill(Color32::from_rgb(248, 250, 252))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                    .corner_radius(8.0)
                                    .inner_margin(egui::Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("▶").size(12.0).color(Color32::from_rgb(79, 70, 229)));
                                            if self.current_track.duration > 0.0 || !self.current_track.title.is_empty() {
                                                let cur_t = if self.current_track.title.is_empty() { "Lagu Aktif".to_string() } else { self.current_track.title.clone() };
                                                let cur_art = if self.current_track.artist.is_empty() { "Artis".to_string() } else { self.current_track.artist.clone() };

                                                ui.label(egui::RichText::new("Sedang Diputar:").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                ui.label(egui::RichText::new(&cur_t).strong().size(11.0).color(Color32::from_rgb(15, 23, 42)));
                                                ui.label(egui::RichText::new(format!("({})", cur_art)).size(10.0).color(Color32::from_rgb(148, 163, 184)));

                                                // Show matched resolution
                                                if let Some((b_type, p_name, _)) = self.evaluate_auto_binding(&self.current_track) {
                                                    let (bg, fg) = b_type.colors();
                                                    Self::render_telemetry_pill(ui, "", b_type.badge_label(), bg, fg);
                                                    Self::render_telemetry_pill(ui, "🎛️", &p_name, Color32::from_rgb(240, 253, 250), Color32::from_rgb(13, 148, 136));
                                                } else if self.binding_fallback_to_bit_perfect {
                                                    Self::render_telemetry_pill(ui, "🛡️", "Bit-Perfect", Color32::from_rgb(241, 245, 249), Color32::from_rgb(71, 85, 105));
                                                }

                                                // Quick action to bind current track
                                                let has_track_bind = self.preset_bindings.iter().any(|b| b.target_type == crate::preset::PresetBindingType::Track && b.target_key.eq_ignore_ascii_case(&cur_t));
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.add(
                                                        egui::Button::new(
                                                            egui::RichText::new(if has_track_bind { "✏️ Ubah Preset Lagu" } else { "🔗 Ikat Lagu Ini" })
                                                                .size(10.0).strong().color(Color32::from_rgb(79, 70, 229))
                                                        )
                                                        .fill(Color32::from_rgb(238, 242, 255))
                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(199, 210, 254)))
                                                        .corner_radius(4.0)
                                                    ).on_hover_text("Buka pemilih untuk menautkan atau memperbarui preset lagu yang sedang aktif diputar").clicked() {
                                                        self.binding_picker_open = true;
                                                        self.binding_picker_mode = 0;
                                                        self.binding_picker_search = cur_t.clone();
                                                        self.binding_picker_selected_key = cur_t.clone();
                                                        self.binding_picker_selected_label = format!("{} - {}", cur_t, cur_art);
                                                    }
                                                });
                                            } else {
                                                ui.label(egui::RichText::new("Belum ada lagu yang sedang aktif diputar di playlist.").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                                            }
                                        });
                                    });

                                ui.add_space(6.0);

                                // 2. Action Toolbar (Filters + Search + Action Buttons)
                                ui.horizontal(|ui| {
                                    // Filter pills
                                    let total_count = self.preset_bindings.len();
                                    let track_count = self.preset_bindings.iter().filter(|b| b.target_type == crate::preset::PresetBindingType::Track).count();
                                    let album_count = self.preset_bindings.iter().filter(|b| b.target_type == crate::preset::PresetBindingType::Album).count();
                                    let genre_count = self.preset_bindings.iter().filter(|b| b.target_type == crate::preset::PresetBindingType::Genre).count();

                                    let f_items = [
                                        (0, format!("Semua ({})", total_count)),
                                        (1, format!("🎵 Trek ({})", track_count)),
                                        (2, format!("💿 Album ({})", album_count)),
                                        (3, format!("🎭 Genre ({})", genre_count)),
                                    ];

                                    for (idx, label) in f_items {
                                        let is_active = self.binding_filter_tab == idx;
                                        let btn = egui::Button::new(
                                            egui::RichText::new(label).size(10.5).strong()
                                                .color(if is_active { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) })
                                        )
                                        .fill(if is_active { Color32::from_rgb(238, 242, 255) } else { Color32::from_rgb(248, 250, 252) })
                                        .stroke(Stroke::new(1.0, if is_active { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                                        .corner_radius(12.0);
                                        if ui.add(btn).clicked() {
                                            self.binding_filter_tab = idx;
                                        }
                                    }

                                    ui.add_space(6.0);

                                    // Search input
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.binding_rule_search)
                                            .hint_text("🔍 Cari aturan...")
                                            .desired_width(130.0)
                                    );
                                    if !self.binding_rule_search.is_empty() {
                                        if ui.add(
                                            egui::Button::new(egui::RichText::new("✕").size(9.0).color(Color32::from_rgb(100, 116, 139)))
                                                .fill(Color32::WHITE)
                                                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                                .corner_radius(4.0)
                                        ).clicked() {
                                            self.binding_rule_search.clear();
                                        }
                                    }

                                    // Right Action Buttons: [ ➕ Tautkan Baru ] & [ ⚙️ Opsi ]
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let opt_active = self.show_binding_options_popover;
                                        let opt_btn = egui::Button::new(
                                            egui::RichText::new("⚙️ Opsi").size(10.5).strong()
                                                .color(if opt_active { Color32::from_rgb(79, 70, 229) } else { Color32::from_rgb(100, 116, 139) })
                                        )
                                        .fill(if opt_active { Color32::from_rgb(238, 242, 255) } else { Color32::WHITE })
                                        .stroke(Stroke::new(1.0, if opt_active { Color32::from_rgb(165, 180, 252) } else { Color32::from_rgb(226, 232, 240) }))
                                        .corner_radius(6.0);
                                        if ui.add(opt_btn).on_hover_text("Buka pengaturan mesin auto-binding dan fallback").clicked() {
                                            self.show_binding_options_popover = !self.show_binding_options_popover;
                                        }

                                        let add_btn = egui::Button::new(
                                            egui::RichText::new("➕ Tautkan Baru...").strong().size(11.0).color(Color32::WHITE)
                                        )
                                        .fill(Color32::from_rgb(79, 70, 229))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(67, 56, 202)))
                                        .corner_radius(6.0);
                                        if ui.add(add_btn).on_hover_text("Pilih lagu, album, atau genre dari koleksi untuk ditautkan").clicked() {
                                            self.binding_picker_open = true;
                                            self.binding_picker_mode = 0;
                                            self.binding_picker_search.clear();
                                            self.binding_picker_selected_key.clear();
                                            self.binding_picker_selected_label.clear();
                                        }
                                    });
                                });

                                // Optional Options Collapsible Card
                                if self.show_binding_options_popover {
                                    ui.add_space(4.0);
                                    egui::Frame::new()
                                        .fill(Color32::from_rgb(241, 245, 249))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                                        .corner_radius(8.0)
                                        .inner_margin(egui::Margin::symmetric(12, 8))
                                        .show(ui, |ui| {
                                            let mut opt_changed = false;
                                            ui.horizontal(|ui| {
                                                if ui.checkbox(&mut self.auto_load_preset_on_track_change, "Otomatis load preset saat ganti lagu").changed() {
                                                    opt_changed = true;
                                                }
                                                ui.add_space(10.0);
                                                if ui.checkbox(&mut self.show_binding_toast_on_change, "Tampilkan notifikasi banner saat profil berganti").changed() {
                                                    opt_changed = true;
                                                }
                                                ui.add_space(10.0);
                                                if ui.checkbox(&mut self.binding_fallback_to_bit_perfect, "🛡️ Fallback ke Bit-Perfect jika tidak ada aturan yang cocok (Weight 0)").changed() {
                                                    opt_changed = true;
                                                }
                                            });
                                            if opt_changed {
                                                self.save_auto_bindings();
                                            }
                                        });
                                }

                                ui.add_space(6.0);

                                // 3. Modern Card List of Active Rules (Scrollable, Full Height, Never Clipped)
                                let search_rule = self.binding_rule_search.trim().to_lowercase();
                                let filtered_rules: Vec<crate::preset::PresetBindingRule> = self.preset_bindings
                                    .iter()
                                    .filter(|b| {
                                        let match_tab = match self.binding_filter_tab {
                                            1 => b.target_type == crate::preset::PresetBindingType::Track,
                                            2 => b.target_type == crate::preset::PresetBindingType::Album,
                                            3 => b.target_type == crate::preset::PresetBindingType::Genre,
                                            _ => true,
                                        };
                                        if !match_tab {
                                            return false;
                                        }
                                        if search_rule.is_empty() {
                                            true
                                        } else {
                                            b.target_label.to_lowercase().contains(&search_rule)
                                                || b.preset_name.to_lowercase().contains(&search_rule)
                                        }
                                    })
                                    .cloned()
                                    .collect();

                                let available_h = ui.available_height().max(180.0);
                                egui::ScrollArea::vertical()
                                    .id_salt("auto_binding_rules_scroll_area")
                                    .auto_shrink([false, false])
                                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                                    .max_height(available_h)
                                    .show(ui, |ui| {
                                        if filtered_rules.is_empty() {
                                            egui::Frame::new()
                                                .fill(Color32::from_rgb(248, 250, 252))
                                                .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                                .corner_radius(8.0)
                                                .inner_margin(egui::Margin::symmetric(20, 24))
                                                .show(ui, |ui| {
                                                    ui.vertical_centered(|ui| {
                                                        ui.label(egui::RichText::new("🔗").size(24.0));
                                                        ui.add_space(4.0);
                                                        ui.label(egui::RichText::new("Belum ada aturan binding dalam kategori ini").strong().size(12.0).color(Color32::from_rgb(71, 85, 105)));
                                                        ui.label(egui::RichText::new("Klik tombol [ ➕ Tautkan Baru... ] di atas untuk mengikat preset ke lagu, album, atau genre.").size(10.5).color(Color32::from_rgb(148, 163, 184)));
                                                    });
                                                });
                                        } else {
                                            for rule in &filtered_rules {
                                                let (bg, fg) = rule.target_type.colors();
                                                let weight_badge = rule.target_type.weight_badge();
                                                let precedence_note = rule.target_type.precedence_note();

                                                egui::Frame::new()
                                                    .fill(Color32::WHITE)
                                                    .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                                                    .corner_radius(8.0)
                                                    .inner_margin(egui::Margin::symmetric(12, 9))
                                                    .show(ui, |ui| {
                                                        ui.horizontal(|ui| {
                                                            // Left Badge & Weight
                                                            ui.vertical(|ui| {
                                                                Self::render_telemetry_pill(ui, "", rule.target_type.badge_label(), bg, fg);
                                                                ui.label(egui::RichText::new(weight_badge).size(9.0).strong().color(Color32::from_rgb(148, 163, 184)));
                                                            });

                                                            ui.add_space(6.0);

                                                            // Center Target Label & Precedence Note
                                                            ui.vertical(|ui| {
                                                                ui.label(egui::RichText::new(&rule.target_label).strong().size(12.0).color(Color32::from_rgb(15, 23, 42)));
                                                                ui.label(egui::RichText::new(precedence_note).size(9.5).color(Color32::from_rgb(100, 116, 139)));
                                                            });

                                                            // Right: Industry Standard Preset Dropdown ComboBox & Delete Button
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                                // Delete button
                                                                if ui.add(
                                                                    egui::Button::new(egui::RichText::new("✕ Lepas").size(10.5).color(Color32::from_rgb(239, 68, 68)))
                                                                        .fill(Color32::WHITE)
                                                                        .stroke(Stroke::new(1.0, Color32::from_rgb(254, 202, 202)))
                                                                        .corner_radius(4.0)
                                                                ).on_hover_text("Hapus aturan binding ini").clicked() {
                                                                    remove_binding_id = Some(rule.id.clone());
                                                                }

                                                                ui.add_space(6.0);

                                                                // Dropdown ComboBox to change preset in-place (Industry Standard)
                                                                let mut cur_preset = rule.preset_name.clone();
                                                                egui::ComboBox::from_id_salt(format!("combo_rule_{}", rule.id))
                                                                    .selected_text(egui::RichText::new(format!("🎛️ {}", cur_preset)).size(10.5).strong().color(Color32::from_rgb(13, 148, 136)))
                                                                    .width(170.0)
                                                                    .show_ui(ui, |ui| {
                                                                        for p_name in &preset_names {
                                                                            if ui.selectable_value(&mut cur_preset, p_name.clone(), p_name).clicked() {
                                                                                update_binding_preset = Some((rule.id.clone(), p_name.clone()));
                                                                            }
                                                                        }
                                                                    });

                                                                ui.label(egui::RichText::new("➔ Muat:").size(10.5).color(Color32::from_rgb(100, 116, 139)));
                                                            });
                                                        });
                                                    });
                                                ui.add_space(4.0);
                                            }
                                        }
                                    });

                                // Handle in-place preset update
                                if let Some((rid, new_preset_name)) = update_binding_preset {
                                    let mut updated_label = String::new();
                                    for b in &mut self.preset_bindings {
                                        if b.id == rid {
                                            b.preset_name = new_preset_name.clone();
                                            updated_label = b.target_label.clone();
                                            break;
                                        }
                                    }
                                    if !updated_label.is_empty() {
                                        self.save_auto_bindings();
                                        self.preset_status_notification = Some((format!("✅ Aturan '{}' diperbarui ke preset '{}'!", updated_label, new_preset_name), true));
                                        self.preset_status_notification_time = Some(std::time::Instant::now());
                                    }
                                }

                                // Handle remove
                                if let Some(ref rid) = remove_binding_id {
                                    self.preset_bindings.retain(|b| &b.id != rid);
                                    self.save_auto_bindings();
                                    self.preset_status_notification = Some(("Aturan auto-binding berhasil dilepas.".to_string(), true));
                                    self.preset_status_notification_time = Some(std::time::Instant::now());
                                }
                            }
                        }
                    });
            });

        // Handle Apply Action from Local Library
        if let Some((path, has_password)) = action_apply_local {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<LnlPresetContainer>(&content) {
                    Ok(container) => {
                        if has_password {
                            self.pending_import_container = Some(container);
                            self.preset_import_password.clear();
                            self.preset_import_error = None;
                            self.show_import_password_modal = true;
                            self.show_export_preset_modal = false;
                        } else {
                            match container.decrypt_and_verify(None) {
                                Ok(preset) => {
                                    let name = preset.name.clone();
                                    self.apply_signal_chain_preset(&preset);
                                    self.preset_status_notification = Some((
                                        format!("✅ Preset '{}' berhasil diterapkan ke rantai sinyal!", name),
                                        true,
                                    ));
                                    self.show_export_preset_modal = false;
                                }
                                Err(e) => {
                                    self.preset_status_notification = Some((format!("❌ Gagal memuat preset: {}", e), false));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.preset_status_notification = Some((format!("❌ Format berkas bukan .lnl yang valid: {}", e), false));
                    }
                },
                Err(e) => {
                    self.preset_status_notification = Some((format!("❌ Gagal membaca berkas preset: {}", e), false));
                }
            }
        }

        // Handle Delete Action from Local Library
        if let Some(path) = action_delete_local {
            let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("preset").to_string();
            if let Ok(()) = std::fs::remove_file(&path) {
                self.preset_status_notification = Some((format!("🗑️ Preset '{}' telah dihapus dari perpustakaan.", file_name), true));
                self.preset_status_notification_time = Some(std::time::Instant::now());
            }
        }

        // Handle Duplicate Action from Local Library
        if let Some(path) = action_duplicate_local {
            match self.duplicate_local_preset(&path) {
                Ok(new_name) => {
                    self.preset_status_notification = Some((
                        format!("📋 Berhasil menduplikasi preset sebagai '{}'!", new_name),
                        true,
                    ));
                    self.preset_status_notification_time = Some(std::time::Instant::now());
                    self.preset_selected_idx = 0;
                }
                Err(e) => {
                    self.preset_status_notification = Some((
                        format!("❌ Gagal menduplikasi preset: {}", e),
                        false,
                    ));
                    self.preset_status_notification_time = Some(std::time::Instant::now());
                }
            }
        }

        // Handle Export Local Preset Copy
        if let Some(path) = action_export_local {
            let default_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("preset.lnl");
            if let Some(dest) = rfd::FileDialog::new()
                .set_title("Ekspor Salinan Berkas Preset (.lnl)")
                .set_file_name(default_name)
                .add_filter("Lunula Studio Preset (*.lnl)", &["lnl"])
                .save_file()
            {
                if let Ok(()) = std::fs::copy(&path, &dest).map(|_| ()) {
                    self.preset_status_notification = Some(("✅ Salinan berkas preset berhasil diekspor!".to_string(), true));
                }
            }
        }

        // Handle Save Directly to Local Collection
        if do_save_to_collection {
            let safe_name = self.preset_export_name.trim();
            let clean_name = if safe_name.is_empty() { "Studio Preset" } else { safe_name };
            let file_name = format!("{}.lnl", clean_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"));
            let dest_path = get_presets_dir().join(file_name);

            let pass = if self.preset_export_use_password && !self.preset_export_password.trim().is_empty() {
                Some(self.preset_export_password.as_str())
            } else {
                None
            };

            match self.save_preset_file(&dest_path, clean_name.to_string(), self.preset_export_author.clone(), self.preset_export_description.clone(), pass) {
                Ok(_) => {
                    self.preset_status_notification = Some((
                        format!("✅ Preset '{}' berhasil disimpan ke koleksi studio!", clean_name),
                        true,
                    ));
                    self.preset_modal_tab = 1; // Switch to collection tab so user sees it right away!
                }
                Err(e) => {
                    self.preset_status_notification = Some((format!("❌ Gagal menyimpan preset: {}", e), false));
                }
            }
        }

        // Handle Export to File
        if do_export_to_file {
            let safe_name = self.preset_export_name.trim();
            let file_name = if safe_name.is_empty() {
                "studio_preset.lnl".to_string()
            } else {
                format!("{}.lnl", safe_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"))
            };

            if let Some(path) = rfd::FileDialog::new()
                .set_title("Simpan Berkas Preset Lunula Studio (.lnl)")
                .set_file_name(&file_name)
                .add_filter("Lunula Studio Preset (*.lnl)", &["lnl"])
                .save_file()
            {
                let pass = if self.preset_export_use_password && !self.preset_export_password.trim().is_empty() {
                    Some(self.preset_export_password.as_str())
                } else {
                    None
                };

                let name = if safe_name.is_empty() { "Studio Preset".to_string() } else { safe_name.to_string() };
                let author = self.preset_export_author.clone();
                let desc = self.preset_export_description.clone();

                match self.save_preset_file(&path, name.clone(), author, desc, pass) {
                    Ok(_) => {
                        self.preset_status_notification = Some((
                            format!("✅ Berkas preset '{}' berhasil diekspor dan ditandatangani (.lnl)!", name),
                            true,
                        ));
                        self.show_export_preset_modal = false;
                    }
                    Err(e) => {
                        self.preset_status_notification = Some((format!("❌ Gagal mengekspor preset: {}", e), false));
                    }
                }
            }
        }

        // Handle Import External File to Collection
        if do_import_to_collection {
            if let Some(path) = rfd::FileDialog::new()
                .set_title("Pilih Berkas Preset .lnl untuk Diimpor ke Koleksi")
                .add_filter("Lunula Studio Preset (*.lnl)", &["lnl"])
                .pick_file()
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(container) = serde_json::from_str::<LnlPresetContainer>(&content) {
                        let safe_name = container.metadata.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                        let dest_path = get_presets_dir().join(format!("{}.lnl", safe_name));
                        if let Ok(()) = std::fs::write(&dest_path, &content) {
                            self.preset_status_notification = Some((
                                format!("✅ Preset '{}' berhasil diimpor ke koleksi studio!", container.metadata.name),
                                true,
                            ));
                        }
                    } else {
                        self.preset_status_notification = Some(("❌ Berkas yang dipilih bukan preset .lnl yang valid".to_string(), false));
                    }
                }
            }
        }

        if close_requested {
            self.show_export_preset_modal = false;
        }
    }

    /// Renders the modal prompt when an imported `.lnl` preset is locked with a custom passphrase.
    pub fn render_import_password_modal(&mut self, ctx: &egui::Context) {
        if !self.show_import_password_modal {
            return;
        }

        let container = match self.pending_import_container.as_ref() {
            Some(c) => c.clone(),
            None => {
                self.show_import_password_modal = false;
                return;
            }
        };

        let mut close_requested = false;
        let mut do_unlock = false;

        let screen_rect = ctx.input(|i| i.raw.screen_rect).unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));
        #[allow(deprecated)]
        let modal_area_id = egui::Id::new("import_preset_password_modal_area");

        egui::Area::new(modal_area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                ui.painter().rect_filled(screen_rect, 0.0, Color32::from_black_alpha(170));

                let backdrop_click = ui.interact(screen_rect, modal_area_id.with("backdrop_click"), egui::Sense::click());
                if backdrop_click.clicked() {
                    close_requested = true;
                }

                let modal_w = 480.0f32.min(screen_rect.width() - 32.0);
                let modal_h = 360.0f32.min(screen_rect.height() - 32.0);
                let modal_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(modal_w, modal_h));

                let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(modal_rect));

                egui::Frame::new()
                    .fill(Color32::WHITE)
                    .stroke(Stroke::new(1.6, Color32::from_rgb(245, 158, 11))) // Amber / Gold Border
                    .corner_radius(16.0)
                    .inner_margin(22.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 20],
                        blur: 48,
                        spread: 0,
                        color: Color32::from_black_alpha(160),
                    })
                    .show(&mut child_ui, |ui| {
                        // Header Bar
                        ui.horizontal(|ui| {
                            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(36.0), egui::Sense::hover());
                            ui.painter().rect_filled(icon_rect, 8.0, Color32::from_rgb(254, 243, 199));
                            ui.painter().text(
                                icon_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🔐",
                                egui::FontId::proportional(18.0),
                                Color32::WHITE,
                            );

                            ui.add_space(8.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("BUKA PRESET TERENKRIPSI (.LNL)")
                                        .strong()
                                        .size(14.5)
                                        .color(Color32::from_rgb(15, 23, 42)),
                                );
                                ui.label(
                                    egui::RichText::new("Verifikasi Kriptografi & Dekripsi Sinyal Studio")
                                        .size(11.0)
                                        .color(Color32::from_rgb(100, 116, 139)),
                                );
                            });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("✕").size(14.0).strong().color(Color32::from_rgb(100, 116, 139)))
                                        .fill(Color32::from_rgb(241, 245, 249))
                                        .corner_radius(14.0)
                                        .min_size(Vec2::splat(28.0)),
                                )
                                .on_hover_text("Batal")
                                .clicked()
                                {
                                    close_requested = true;
                                }
                            });
                        });

                        ui.add_space(10.0);

                        // Preset Information Badge Card
                        egui::Frame::new()
                            .fill(Color32::from_rgb(248, 250, 252))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(226, 232, 240)))
                            .corner_radius(8.0)
                            .inner_margin(egui::Margin::symmetric(12, 10))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&container.metadata.name)
                                                .strong()
                                                .size(13.0)
                                                .color(Color32::from_rgb(79, 70, 229)),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!("oleh {}", container.metadata.author))
                                                .size(11.0)
                                                .color(Color32::from_rgb(100, 116, 139)),
                                        );
                                    });

                                    if !container.metadata.description.is_empty() {
                                        ui.add_space(2.0);
                                        ui.label(
                                            egui::RichText::new(&container.metadata.description)
                                                .size(10.5)
                                                .color(Color32::from_rgb(100, 116, 139)),
                                        );
                                    }

                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        Self::render_telemetry_pill(
                                            ui,
                                            "🎛️",
                                            &format!("{} Modul", container.metadata.node_count),
                                            Color32::from_rgb(238, 242, 255),
                                            Color32::from_rgb(79, 70, 229),
                                        );
                                        Self::render_telemetry_pill(
                                            ui,
                                            "🔌",
                                            &format!("{} Kabel", container.metadata.connection_count),
                                            Color32::from_rgb(240, 253, 244),
                                            Color32::from_rgb(22, 163, 74),
                                        );
                                    });
                                });
                            });

                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("🔑 KATA SANDI PRESET:").strong().size(11.0).color(Color32::from_rgb(71, 85, 105)));
                        ui.add_space(3.0);

                        let pass_field = egui::Frame::new()
                            .fill(Color32::from_rgb(248, 250, 252))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(203, 213, 225)))
                            .corner_radius(6.0)
                            .inner_margin(egui::Margin::symmetric(10, 7))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("🔑").size(12.0));
                                    ui.add_space(2.0);
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.preset_import_password)
                                            .password(true)
                                            .hint_text("Ketik kata sandi untuk mendekripsi...")
                                            .frame(egui::Frame::NONE)
                                            .desired_width(ui.available_width()),
                                    )
                                }).inner
                            }).inner;

                        if pass_field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            do_unlock = true;
                        }

                        if let Some(ref err) = self.preset_import_error {
                            ui.add_space(6.0);
                            egui::Frame::new()
                                .fill(Color32::from_rgb(254, 242, 242))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(254, 202, 202)))
                                .corner_radius(6.0)
                                .inner_margin(egui::Margin::symmetric(10, 6))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(format!("❌ {}", err)).size(11.0).color(Color32::from_rgb(185, 28, 28)));
                                });
                        }

                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            if ui.add(
                                egui::Button::new(egui::RichText::new("Batal").size(11.5).color(Color32::from_rgb(100, 116, 139)))
                                    .fill(Color32::from_rgb(241, 245, 249))
                                    .corner_radius(6.0)
                                    .min_size(Vec2::new(70.0, 32.0)),
                            )
                            .clicked()
                            {
                                close_requested = true;
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(
                                    egui::Button::new(egui::RichText::new("🔓 Buka & Terapkan").strong().size(11.5).color(Color32::WHITE))
                                        .fill(Color32::from_rgb(79, 70, 229))
                                        .corner_radius(6.0)
                                        .min_size(Vec2::new(140.0, 32.0)),
                                )
                                .clicked()
                                {
                                    do_unlock = true;
                                }
                            });
                        });
                    });
            });

        if do_unlock {
            match container.decrypt_and_verify(Some(&self.preset_import_password)) {
                Ok(preset) => {
                    let name = preset.name.clone();
                    self.apply_signal_chain_preset(&preset);
                    self.preset_status_notification = Some((
                        format!("✅ Preset '{}' berhasil dibuka dan diterapkan ke studio!", name),
                        true,
                    ));
                    self.show_import_password_modal = false;
                    self.pending_import_container = None;
                    self.preset_import_password.clear();
                    self.preset_import_error = None;
                }
                Err(e) => {
                    self.preset_import_error = Some(format!("{}", e));
                }
            }
        }

        if close_requested {
            self.show_import_password_modal = false;
            self.pending_import_container = None;
            self.preset_import_password.clear();
            self.preset_import_error = None;
        }
    }
}

// ============================================================================
// 5. UNIT TESTS FOR CRYPTOGRAPHY & PRESET FORMAT
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_nist_standard_vector() {
        let hash = sha256(b"abc");
        assert_eq!(
            hex_encode(&hash),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_hmac_sha256_rfc_vector() {
        let key = b"key";
        let data = b"The quick brown fox jumps over the lazy dog";
        let mac = hmac_sha256(key, data);
        assert_eq!(
            hex_encode(&mac),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn test_chacha20_roundtrip() {
        let key = [0x42u8; 32];
        let nonce = [0x24u8; 12];
        let original = b"Lunula High Fidelity 64-bit Audio DSP Chain Preset Payload".to_vec();

        let mut cipher = original.clone();
        chacha20_xor(&key, &nonce, 1, &mut cipher);
        assert_ne!(cipher, original);

        chacha20_xor(&key, &nonce, 1, &mut cipher);
        assert_eq!(cipher, original);
    }

    #[test]
    fn test_base64_roundtrip() {
        let input = b"Lunula .lnl Encrypted JSON Signal Chain Preset";
        let encoded = base64_encode(input);
        let decoded = base64_decode(&encoded).expect("base64 decode");
        assert_eq!(input.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_preset_lnl_export_and_import_roundtrip() {
        let preset = SignalChainPreset {
            name: "Warm Tube & Spatial Wide".to_string(),
            author: "Acoustic Engineer".to_string(),
            description: "Custom vintage signal chain for smooth vocals".to_string(),
            created_at: 1794163000,
            app_version: "1.0.1".to_string(),
            chain_mode: ChainMode::DspActive,
            studio_mode: StudioMode::Advance,
            nodes: vec![
                PresetNode {
                    id: 1,
                    node_type: NodeType::Source,
                    title: "FILE SOURCE".to_string(),
                    sub1: "FLAC".to_string(),
                    sub2: "96kHz".to_string(),
                    pos: [30.0, 60.0],
                    is_removable: false,
                    param_val: 0.0,
                },
                PresetNode {
                    id: 6,
                    node_type: NodeType::TubeWarmer,
                    title: "Tube Warmer".to_string(),
                    sub1: "Vintage Tube".to_string(),
                    sub2: "45% Drive".to_string(),
                    pos: [350.0, 120.0],
                    is_removable: true,
                    param_val: 45.0,
                },
            ],
            connections: vec![
                CableConnection::new(1, 6),
                CableConnection::new(6, 4),
            ],
            active_eq: None,
            advance_studio_dither_mode: 1,
        };

        // Export to .lnl JSON string
        let lnl_str = preset.to_lnl_string(None).expect("export to .lnl");
        assert!(lnl_str.contains("\"magic\": \"LNLPRESET\""));
        assert!(lnl_str.contains("\"algorithm\": \"CHACHA20-HMAC-SHA256\""));

        // Import and verify
        let restored = LnlPresetContainer::from_lnl_string(&lnl_str, None).expect("import from .lnl");
        assert_eq!(restored.name, preset.name);
        assert_eq!(restored.author, preset.author);
        assert_eq!(restored.nodes.len(), 2);
        assert_eq!(restored.nodes[1].node_type, NodeType::TubeWarmer);
        assert_eq!(restored.nodes[1].param_val, 45.0);
        assert_eq!(restored.connections.len(), 2);
    }

    #[test]
    fn test_preset_lnl_password_protection() {
        let preset = SignalChainPreset {
            name: "Private Studio Preset".to_string(),
            author: "Mastering Studio".to_string(),
            description: "Encrypted with client passphrase".to_string(),
            created_at: 1794163000,
            app_version: "1.0.1".to_string(),
            chain_mode: ChainMode::DspActive,
            studio_mode: StudioMode::Simple,
            nodes: vec![],
            connections: vec![],
            active_eq: None,
            advance_studio_dither_mode: 0,
        };

        let password = "MySecretStudioPassword!2026";
        let lnl_str = preset.to_lnl_string(Some(password)).expect("export password protected");

        // Decrypt with correct password
        let ok = LnlPresetContainer::from_lnl_string(&lnl_str, Some(password));
        assert!(ok.is_ok());

        // Decrypt with WRONG password -> must fail signature verification!
        let wrong = LnlPresetContainer::from_lnl_string(&lnl_str, Some("WrongPassword123"));
        assert!(matches!(wrong, Err(PresetError::InvalidSignature)));

        // Decrypt with NO password -> must fail signature verification!
        let no_pass = LnlPresetContainer::from_lnl_string(&lnl_str, None);
        assert!(matches!(no_pass, Err(PresetError::InvalidSignature)));
    }

    #[test]
    fn test_preset_lnl_tamper_detection() {
        let preset = SignalChainPreset {
            name: "Tamper Test".to_string(),
            author: "Tester".to_string(),
            description: "Test".to_string(),
            created_at: 1794163000,
            app_version: "1.0.1".to_string(),
            chain_mode: ChainMode::BitPerfect,
            studio_mode: StudioMode::Simple,
            nodes: vec![],
            connections: vec![],
            active_eq: None,
            advance_studio_dither_mode: 0,
        };

        let container = preset.to_container(None).expect("container");
        let mut tampered = container.clone();

        // Tamper with payload
        let mut bytes = base64_decode(&tampered.payload_base64).unwrap();
        if let Some(b) = bytes.first_mut() {
            *b ^= 0x01; // flip 1 bit
        }
        tampered.payload_base64 = base64_encode(&bytes);

        // Signature check must detect tampering!
        let res = tampered.decrypt_and_verify(None);
        assert!(matches!(res, Err(PresetError::InvalidSignature)));
    }

    #[test]
    fn test_app_capture_and_apply_lnl_preset_roundtrip() {
        let mut app = LunulaApp::new_standalone();
        app.add_modular_filter(NodeType::TubeWarmer);
        app.add_modular_filter(NodeType::LowPassFilter);
        app.set_chain_mode(ChainMode::DspActive);
        app.studio_mode = StudioMode::Advance;

        // Capture preset
        let preset = app.capture_current_signal_chain_preset(
            "Studio Test Preset".to_string(),
            "Unit Test".to_string(),
            "Integration Test Preset".to_string(),
        );

        let lnl_str = preset.to_lnl_string(None).expect("to lnl");

        // Reset app to clean state
        let mut fresh_app = LunulaApp::new_standalone();
        assert_eq!(fresh_app.nodes.len(), 4); // default 4 nodes
        assert_eq!(fresh_app.studio_mode, StudioMode::Simple);

        // Load & apply preset
        let loaded = LnlPresetContainer::from_lnl_string(&lnl_str, None).expect("load lnl");
        fresh_app.apply_signal_chain_preset(&loaded);

        assert_eq!(fresh_app.nodes.len(), 6);
        assert_eq!(fresh_app.studio_mode, StudioMode::Advance);
        assert_eq!(fresh_app.chain_mode, ChainMode::DspActive);
        assert!(fresh_app.nodes.iter().any(|n| n.node_type == NodeType::TubeWarmer));
        assert!(fresh_app.nodes.iter().any(|n| n.node_type == NodeType::LowPassFilter));
    }

    #[test]
    fn test_render_preset_modals_headless() {
        let mut app = LunulaApp::new_standalone();
        let ctx = egui::Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));

        // 1. Render Tab 0 (Simpan Baru)
        app.show_export_preset_modal = true;
        app.preset_modal_tab = 0;
        let mut out0 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ui| {
                app.render_export_preset_modal(ui.ctx());
            },
        );
        out0.textures_delta.clear();

        // 2. Render Tab 1 (Perpustakaan Preset)
        app.preset_modal_tab = 1;
        let mut out1 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ui| {
                app.render_export_preset_modal(ui.ctx());
            },
        );
        out1.textures_delta.clear();

        // 3. Render Tab 2 (Auto-Binding Prototype)
        app.preset_modal_tab = 2;
        let mut out_tab2 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ui| {
                app.render_export_preset_modal(ui.ctx());
            },
        );
        out_tab2.textures_delta.clear();

        // 4. Render Password Import Modal
        app.show_import_password_modal = true;
        let dummy_preset = app.capture_current_signal_chain_preset(
            "Locked Preset".to_string(),
            "Author".to_string(),
            "Secret".to_string(),
        );
        let container = dummy_preset.to_container(Some("secret123")).expect("container");
        app.pending_import_container = Some(container);

        let mut out2 = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ui| {
                app.render_import_password_modal(ui.ctx());
            },
        );
        out2.textures_delta.clear();

        assert_eq!(app.preset_modal_tab, 2);
    }

    #[test]
    fn test_render_mini_canvas_preview() {
        let app = LunulaApp::new_standalone();
        let ctx = egui::Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));

        let preset = app.capture_current_signal_chain_preset(
            "Test Preview Preset".to_string(),
            "Lunula Test".to_string(),
            "Testing mini-canvas preview".to_string(),
        );

        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: screen,
                ..Default::default()
            },
            |ui| {
                LunulaApp::render_mini_canvas_preview(ui, &preset, Vec2::new(450.0, 240.0));
            },
        );
        out.textures_delta.clear();
        assert!(!out.shapes.is_empty(), "Mini canvas preview must generate shapes (dot grid, cables, nodes)");
    }

    #[test]
    fn test_auto_binding_evaluation_cascading_priority() {
        use crate::models::Track;

        let mut app = LunulaApp::new_standalone();
        app.preset_bindings = vec![
            PresetBindingRule {
                id: "rule_t1".to_string(),
                target_type: PresetBindingType::Track,
                target_key: "Bohemian Rhapsody".to_string(),
                target_label: "Track Bohemian Rhapsody".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
            PresetBindingRule {
                id: "rule_a1".to_string(),
                target_type: PresetBindingType::Album,
                target_key: "A Night at the Opera".to_string(),
                target_label: "Album Night at the Opera".to_string(),
                preset_name: "Acoustic Vocal Intimacy".to_string(),
            },
            PresetBindingRule {
                id: "rule_g1".to_string(),
                target_type: PresetBindingType::Genre,
                target_key: "Rock".to_string(),
                target_label: "Genre Rock".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
        ];

        // 1. Priority 1 Test: Exact track match should take precedence over album and genre
        let track1 = Track {
            title: "bohemian rhapsody".to_string(), // case-insensitive
            artist: "Queen".to_string(),
            album: "A Night at the Opera".to_string(),
            genre: Some("Rock".to_string()),
            ..Default::default()
        };
        let eval1 = app.evaluate_auto_binding(&track1);
        assert!(eval1.is_some());
        let (b_type, p_name, _) = eval1.unwrap();
        assert_eq!(b_type, PresetBindingType::Track);
        assert_eq!(p_name, "Warm Studio Master");

        // 2. Priority 2 Test: Different track title on same album should match Album
        let track2 = Track {
            title: "Love of My Life".to_string(),
            artist: "Queen".to_string(),
            album: "a night at the opera".to_string(), // case-insensitive
            genre: Some("Rock".to_string()),
            ..Default::default()
        };
        let eval2 = app.evaluate_auto_binding(&track2);
        assert!(eval2.is_some());
        let (b_type2, p_name2, _) = eval2.unwrap();
        assert_eq!(b_type2, PresetBindingType::Album);
        assert_eq!(p_name2, "Acoustic Vocal Intimacy");

        // 3. Priority 3 Test: Different album with same genre should match Genre
        let track3 = Track {
            title: "Back in Black".to_string(),
            artist: "AC/DC".to_string(),
            album: "Back in Black".to_string(),
            genre: Some("rock".to_string()), // case-insensitive
            ..Default::default()
        };
        let eval3 = app.evaluate_auto_binding(&track3);
        assert!(eval3.is_some());
        let (b_type3, p_name3, _) = eval3.unwrap();
        assert_eq!(b_type3, PresetBindingType::Genre);
        assert_eq!(p_name3, "Warm Studio Master");

        // 4. Priority 4 Test: Non-matching track returns None
        let track4 = Track {
            title: "Gymnopedie No. 1".to_string(),
            artist: "Erik Satie".to_string(),
            album: "Classical Chill".to_string(),
            genre: Some("Classical".to_string()),
            ..Default::default()
        };
        let eval4 = app.evaluate_auto_binding(&track4);
        assert!(eval4.is_none());
    }

    #[test]
    fn test_auto_binding_execution_and_fallback_to_bit_perfect() {
        use crate::models::Track;

        let mut app = LunulaApp::new_standalone();
        app.auto_load_preset_on_track_change = true;
        app.show_binding_toast_on_change = true;
        app.binding_fallback_to_bit_perfect = true;
        app.preset_bindings = vec![
            PresetBindingRule {
                id: "rule_acoustic".to_string(),
                target_type: PresetBindingType::Genre,
                target_key: "Jazz".to_string(),
                target_label: "Jazz Genre".to_string(),
                preset_name: "Acoustic Vocal Intimacy".to_string(),
            },
        ];

        // Start in DSP Active mode
        app.chain_mode = ChainMode::DspActive;

        // Play an unmatched track -> Should fallback to BitPerfect!
        let pop_track = Track {
            title: "Random Song".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Single".to_string(),
            genre: Some("Synthpop".to_string()),
            ..Default::default()
        };
        app.check_and_apply_auto_binding(&pop_track);
        assert_eq!(app.chain_mode, ChainMode::BitPerfect, "Unmatched track must fallback to Bit-Perfect mode");
        assert!(app.preset_status_notification.is_some());
        assert!(app.preset_status_notification.as_ref().unwrap().0.contains("Fallback ke Bit-Perfect"));

        // Now play a matched track -> Should apply the preset!
        let jazz_track = Track {
            title: "Autumn Leaves".to_string(),
            artist: "Miles Davis".to_string(),
            album: "Somethin' Else".to_string(),
            genre: Some("Jazz".to_string()),
            ..Default::default()
        };
        app.check_and_apply_auto_binding(&jazz_track);
        assert_eq!(app.chain_mode, ChainMode::DspActive);
        assert!(app.preset_status_notification.is_some());
        assert!(app.preset_status_notification.as_ref().unwrap().0.contains("Acoustic Vocal Intimacy"));

        // If fallback is disabled, an unmatched track keeps the previous mode
        app.binding_fallback_to_bit_perfect = false;
        app.check_and_apply_auto_binding(&pop_track);
        assert_eq!(app.chain_mode, ChainMode::DspActive, "Without fallback enabled, chain mode remains untouched");
    }

    #[test]
    fn test_auto_binding_storage_roundtrip() {
        let mut app = LunulaApp::new_standalone();
        app.auto_load_preset_on_track_change = true;
        app.show_binding_toast_on_change = true;
        app.binding_fallback_to_bit_perfect = true;
        app.preset_bindings = vec![
            PresetBindingRule {
                id: "test_rule_1".to_string(),
                target_type: PresetBindingType::Track,
                target_key: "Song 1".to_string(),
                target_label: "Track: Song 1".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
        ];

        let storage = AutoBindingStorage {
            auto_load_enabled: app.auto_load_preset_on_track_change,
            show_toast: app.show_binding_toast_on_change,
            fallback_to_bit_perfect: app.binding_fallback_to_bit_perfect,
            bindings: app.preset_bindings.clone(),
        };

        let serialized = serde_json::to_string_pretty(&storage).expect("serialize AutoBindingStorage");
        let deserialized: AutoBindingStorage = serde_json::from_str(&serialized).expect("deserialize AutoBindingStorage");

        assert_eq!(deserialized.auto_load_enabled, true);
        assert_eq!(deserialized.show_toast, true);
        assert_eq!(deserialized.fallback_to_bit_perfect, true);
        assert_eq!(deserialized.bindings.len(), 1);
        assert_eq!(deserialized.bindings[0].target_key, "Song 1");
    }

    #[test]
    fn test_get_preset_binding_targets() {
        let mut app = LunulaApp::new_standalone();
        app.preset_bindings = vec![
            PresetBindingRule {
                id: "1".to_string(),
                target_type: PresetBindingType::Track,
                target_key: "Song A".to_string(),
                target_label: "Track A".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
            PresetBindingRule {
                id: "2".to_string(),
                target_type: PresetBindingType::Track,
                target_key: "Song B".to_string(),
                target_label: "Track B".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
            PresetBindingRule {
                id: "3".to_string(),
                target_type: PresetBindingType::Album,
                target_key: "Album X".to_string(),
                target_label: "Album X".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
            PresetBindingRule {
                id: "4".to_string(),
                target_type: PresetBindingType::Genre,
                target_key: "Jazz".to_string(),
                target_label: "Jazz".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
        ];

        let (tracks, albums, genres) = app.get_preset_binding_targets("warm studio master");
        assert_eq!(tracks.len(), 2);
        assert_eq!(albums.len(), 1);
        assert_eq!(genres.len(), 1);
        assert_eq!(tracks[0], "Song A");
        assert_eq!(tracks[1], "Song B");
        assert_eq!(albums[0], "Album X");
        assert_eq!(genres[0], "Jazz");

        // Unbound preset returns empty lists
        let (un_t, un_a, un_g) = app.get_preset_binding_targets("Non-Existent Preset");
        assert!(un_t.is_empty());
        assert!(un_a.is_empty());
        assert!(un_g.is_empty());
    }

    #[test]
    fn test_duplicate_local_preset() {
        let mut app = LunulaApp::new_standalone();
        ensure_default_factory_presets();

        let dir = get_presets_dir();
        let original_file = dir.join("Warm Studio Master.lnl");
        assert!(original_file.exists(), "Factory preset file must exist");

        // First duplicate
        let dup1_res = app.duplicate_local_preset(&original_file);
        assert!(dup1_res.is_ok(), "Duplication 1 should succeed: {:?}", dup1_res.err());
        let dup1_name = dup1_res.unwrap();
        assert_eq!(dup1_name, "Warm Studio Master (Salinan)");
        let dup1_file = dir.join(format!("{}.lnl", dup1_name));
        assert!(dup1_file.exists());

        // Second duplicate should increment to (Salinan 2)
        let dup2_res = app.duplicate_local_preset(&original_file);
        assert!(dup2_res.is_ok(), "Duplication 2 should succeed: {:?}", dup2_res.err());
        let dup2_name = dup2_res.unwrap();
        assert_eq!(dup2_name, "Warm Studio Master (Salinan 2)");
        let dup2_file = dir.join(format!("{}.lnl", dup2_name));
        assert!(dup2_file.exists());

        // Verify the duplicated file is a valid decryptable container
        let content = std::fs::read_to_string(&dup1_file).expect("read dup1");
        let container: LnlPresetContainer = serde_json::from_str(&content).expect("parse dup1 container");
        assert_eq!(container.metadata.name, "Warm Studio Master (Salinan)");
        let decrypted = container.decrypt_and_verify(None);
        assert!(decrypted.is_ok());
        assert_eq!(decrypted.unwrap().name, "Warm Studio Master (Salinan)");

        // Clean up test duplicates
        let _ = std::fs::remove_file(dup1_file);
        let _ = std::fs::remove_file(dup2_file);
    }

    #[test]
    fn test_visual_library_picker_and_tab2_headless_render() {
        let mut app = LunulaApp::new_standalone();
        let ctx = egui::Context::default();
        let screen = Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 800.0)));

        // Populate mock playlist
        app.playlist = vec![
            crate::models::Track {
                title: "Midnight City".to_string(),
                artist: "M83".to_string(),
                album: "Hurry Up, We're Dreaming".to_string(),
                duration: 243.0,
                format: "FLAC".to_string(),
                sample_rate: 44100,
                bit_depth: 16,
                waveform: vec![],
                waveform_bass: vec![],
                waveform_mid: vec![],
                waveform_treble: vec![],
                file_path: Some("C:/Music/M83 - Midnight City.flac".to_string()),
                file_size_mb: 28.5,
                bitrate_kbps: Some(920),
                is_favorite: false,
                replay_gain: Default::default(),
                cue_offset_sec: None,
                cue_duration_sec: None,
                genre: Some("Synthpop".to_string()),
            },
            crate::models::Track {
                title: "So What".to_string(),
                artist: "Miles Davis".to_string(),
                album: "Kind of Blue".to_string(),
                duration: 562.0,
                format: "FLAC".to_string(),
                sample_rate: 96000,
                bit_depth: 24,
                waveform: vec![],
                waveform_bass: vec![],
                waveform_mid: vec![],
                waveform_treble: vec![],
                file_path: Some("C:/Music/Miles Davis - So What.flac".to_string()),
                file_size_mb: 65.2,
                bitrate_kbps: Some(1500),
                is_favorite: true,
                replay_gain: Default::default(),
                cue_offset_sec: None,
                cue_duration_sec: None,
                genre: Some("Jazz".to_string()),
            },
        ];

        app.show_export_preset_modal = true;
        app.preset_modal_tab = 2;

        // 1. Render Tab 2 with binding_picker_open = false (Rules Overview & Launcher)
        app.binding_picker_open = false;
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_export_preset_modal(ui.ctx()); },
        );
        out.textures_delta.clear();

        // 2. Render Tab 2 with binding_picker_open = true, Mode 0 (Trek)
        app.binding_picker_open = true;
        app.binding_picker_mode = 0;
        app.binding_picker_search = "Mid".to_string();
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_export_preset_modal(ui.ctx()); },
        );
        out.textures_delta.clear();

        // 3. Render Tab 2 with binding_picker_open = true, Mode 1 (Album)
        app.binding_picker_mode = 1;
        app.binding_picker_search = "Kind".to_string();
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_export_preset_modal(ui.ctx()); },
        );
        out.textures_delta.clear();

        // 4. Render Tab 2 with binding_picker_open = true, Mode 2 (Genre)
        app.binding_picker_mode = 2;
        app.binding_picker_search = "Jazz".to_string();
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_export_preset_modal(ui.ctx()); },
        );
        out.textures_delta.clear();

        // 5. Verify binding creation logic programmatically matching picker behavior
        app.preset_bindings.clear();
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "test_picker_bind_1".to_string(),
            target_type: crate::preset::PresetBindingType::Track,
            target_key: "Midnight City".to_string(),
            target_label: "Midnight City - M83".to_string(),
            preset_name: "Warm Studio Master".to_string(),
        });
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "test_picker_bind_2".to_string(),
            target_type: crate::preset::PresetBindingType::Album,
            target_key: "Kind of Blue".to_string(),
            target_label: "Album: Kind of Blue".to_string(),
            preset_name: "Transparent Mastering".to_string(),
        });

        // Test track evaluation
        let eval_track = app.evaluate_auto_binding(&app.playlist[0]);
        assert!(eval_track.is_some());
        let (t_type, p_name, _) = eval_track.unwrap();
        assert_eq!(t_type, crate::preset::PresetBindingType::Track);
        assert_eq!(p_name, "Warm Studio Master");

        // Test album evaluation
        let eval_album = app.evaluate_auto_binding(&app.playlist[1]);
        assert!(eval_album.is_some());
        let (t_type, p_name, _) = eval_album.unwrap();
        assert_eq!(t_type, crate::preset::PresetBindingType::Album);
        assert_eq!(p_name, "Transparent Mastering");

        // 6. Test Weight helper methods
        assert_eq!(crate::preset::PresetBindingType::Track.weight(), 300);
        assert_eq!(crate::preset::PresetBindingType::Album.weight(), 200);
        assert_eq!(crate::preset::PresetBindingType::Genre.weight(), 100);

        // 7. Test Strict Overwrite: One track can only have 1 individual preset rule
        let key_overwrite = "Midnight City".to_string();
        app.preset_bindings.retain(|b| !(b.target_type == crate::preset::PresetBindingType::Track && b.target_key.eq_ignore_ascii_case(&key_overwrite)));
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "test_overwrite_rule".to_string(),
            target_type: crate::preset::PresetBindingType::Track,
            target_key: key_overwrite.clone(),
            target_label: "Midnight City - M83".to_string(),
            preset_name: "Transparent Mastering".to_string(),
        });
        let track_rules: Vec<_> = app.preset_bindings.iter().filter(|b| b.target_type == crate::preset::PresetBindingType::Track && b.target_key.eq_ignore_ascii_case(&key_overwrite)).collect();
        assert_eq!(track_rules.len(), 1, "Only 1 individual track rule must exist after overwrite");
        assert_eq!(track_rules[0].preset_name, "Transparent Mastering");

        // 8. Test 3-way Conflict Precedence (Track vs Album vs Genre on same song)
        app.preset_bindings.clear();
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "rule_t".to_string(),
            target_type: crate::preset::PresetBindingType::Track,
            target_key: "Midnight City".to_string(),
            target_label: "Midnight City - M83".to_string(),
            preset_name: "Preset_Track_Winner".to_string(),
        });
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "rule_a".to_string(),
            target_type: crate::preset::PresetBindingType::Album,
            target_key: "Hurry Up, We're Dreaming".to_string(),
            target_label: "Album: Hurry Up".to_string(),
            preset_name: "Preset_Album_RunnerUp".to_string(),
        });
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "rule_g".to_string(),
            target_type: crate::preset::PresetBindingType::Genre,
            target_key: "Synthpop".to_string(),
            target_label: "Genre: Synthpop".to_string(),
            preset_name: "Preset_Genre_Base".to_string(),
        });

        // Track has all 3 bindings: Track rule MUST win (Weight 300)
        let eval_3way = app.evaluate_auto_binding(&app.playlist[0]).unwrap();
        assert_eq!(eval_3way.0, crate::preset::PresetBindingType::Track);
        assert_eq!(eval_3way.1, "Preset_Track_Winner");

        // Remove track rule: Album rule MUST win (Weight 200)
        app.preset_bindings.retain(|b| b.id != "rule_t");
        let eval_2way = app.evaluate_auto_binding(&app.playlist[0]).unwrap();
        assert_eq!(eval_2way.0, crate::preset::PresetBindingType::Album);
        assert_eq!(eval_2way.1, "Preset_Album_RunnerUp");

        // Remove album rule: Genre rule MUST win (Weight 100)
        app.preset_bindings.retain(|b| b.id != "rule_a");
        let eval_1way = app.evaluate_auto_binding(&app.playlist[0]).unwrap();
        assert_eq!(eval_1way.0, crate::preset::PresetBindingType::Genre);
        assert_eq!(eval_1way.1, "Preset_Genre_Base");

        // Remove genre rule: Evaluates to None -> Triggers Bit-Perfect Fallback (Weight 0)
        app.preset_bindings.retain(|b| b.id != "rule_g");
        let eval_none = app.evaluate_auto_binding(&app.playlist[0]);
        assert!(eval_none.is_none());
    }

    #[test]
    fn test_step3_library_context_menu_bindings_and_visual_badges() {
        let ctx = egui::Context::default();
        let screen = Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 850.0)));
        let mut app = LunulaApp::default();

        // 1. Setup sample tracks in library
        let track_a = crate::models::Track {
            title: "Hikaru Nara".to_string(),
            artist: "Goose house".to_string(),
            album: "Shigatsu wa Kimi no Uso".to_string(),
            duration: 250.0,
            format: "FLAC".to_string(),
            sample_rate: 96000,
            bit_depth: 24,
            waveform: vec![0.5; 10],
            waveform_bass: vec![0.5; 10],
            waveform_mid: vec![0.5; 10],
            waveform_treble: vec![0.5; 10],
            file_path: Some("C:/music/hikaru.flac".to_string()),
            file_size_mb: 45.0,
            bitrate_kbps: None,
            is_favorite: true,
            replay_gain: Default::default(),
            cue_offset_sec: None,
            cue_duration_sec: None,
            genre: Some("J-Pop".to_string()),
        };
        let track_b = crate::models::Track {
            title: "Kirameki".to_string(),
            artist: "wacci".to_string(),
            album: "Shigatsu wa Kimi no Uso".to_string(),
            duration: 270.0,
            format: "FLAC".to_string(),
            sample_rate: 44100,
            bit_depth: 16,
            waveform: vec![0.5; 10],
            waveform_bass: vec![0.5; 10],
            waveform_mid: vec![0.5; 10],
            waveform_treble: vec![0.5; 10],
            file_path: Some("C:/music/kirameki.flac".to_string()),
            file_size_mb: 28.0,
            bitrate_kbps: None,
            is_favorite: false,
            replay_gain: Default::default(),
            cue_offset_sec: None,
            cue_duration_sec: None,
            genre: Some("Ballad".to_string()),
        };
        app.playlist = vec![track_a.clone(), track_b.clone()];

        // 2. Add an Album binding rule for "Shigatsu wa Kimi no Uso"
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "bind_alb_test".to_string(),
            target_type: crate::preset::PresetBindingType::Album,
            target_key: "Shigatsu wa Kimi no Uso".to_string(),
            target_label: "Album: Shigatsu wa Kimi no Uso".to_string(),
            preset_name: "Warm Studio Master".to_string(),
        });

        // 3. Add an individual Track binding rule for "Hikaru Nara"
        app.preset_bindings.push(crate::preset::PresetBindingRule {
            id: "bind_trk_test".to_string(),
            target_type: crate::preset::PresetBindingType::Track,
            target_key: "Hikaru Nara".to_string(),
            target_label: "Goose house - Hikaru Nara".to_string(),
            preset_name: "Acoustic Vocal Intimacy".to_string(),
        });

        // Verify evaluation:
        // track_a has both Album & Track rules -> Track wins (Weight 300)
        let eval_a = app.evaluate_auto_binding(&track_a).unwrap();
        assert_eq!(eval_a.0, crate::preset::PresetBindingType::Track);
        assert_eq!(eval_a.1, "Acoustic Vocal Intimacy");

        // track_b has only Album rule -> Album wins (Weight 200)
        let eval_b = app.evaluate_auto_binding(&track_b).unwrap();
        assert_eq!(eval_b.0, crate::preset::PresetBindingType::Album);
        assert_eq!(eval_b.1, "Warm Studio Master");

        // 4. Render AllTracks Category Headless (verifies badge layout & zero crash)
        app.library_category = crate::models::LibraryCategory::AllTracks;
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_library_panel(ui); },
        );
        out.textures_delta.clear();

        // 5. Render Favorites Category Headless (verifies badge layout on track_a)
        app.library_category = crate::models::LibraryCategory::Favorites;
        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |ui| { app.render_library_panel(ui); },
        );
        out.textures_delta.clear();

        // 6. Test Luxury Toast Notification Banner
        app.preset_status_notification = Some((
            "🔗 Berhasil menautkan trek 'Hikaru Nara' ke Preset Studio 'Acoustic Vocal Intimacy' (Weight: 300)!".to_string(),
            true,
        ));
        app.preset_status_notification_time = Some(std::time::Instant::now());

        let mut out = ctx.run_ui(
            egui::RawInput { screen_rect: screen, ..Default::default() },
            |_ui| {
                if let Some((ref msg, is_success)) = app.preset_status_notification {
                    assert!(is_success);
                    assert!(msg.contains("Hikaru Nara"));
                }
            },
        );
        out.textures_delta.clear();

        // 7. Test unbinding
        let key_unbind = "Hikaru Nara".to_string();
        app.preset_bindings.retain(|b| !(b.target_type == crate::preset::PresetBindingType::Track && b.target_key.eq_ignore_ascii_case(&key_unbind)));
        // After unbinding track rule, track_a should now inherit the Album rule!
        let eval_a_after = app.evaluate_auto_binding(&track_a).unwrap();
        assert_eq!(eval_a_after.0, crate::preset::PresetBindingType::Album);
        assert_eq!(eval_a_after.1, "Warm Studio Master");
    }

    #[test]
    fn test_startup_preset_applied_immediately_and_quietly() {
        let mut app = LunulaApp::new_standalone();
        app.auto_load_preset_on_track_change = true;
        app.show_binding_toast_on_change = false; // default is silent

        let track = crate::models::Track {
            title: "Acoustic Morning".to_string(),
            artist: "Guitarist".to_string(),
            album: "Peaceful Days".to_string(),
            genre: Some("Acoustic".to_string()),
            ..Default::default()
        };
        app.current_track = track.clone();

        app.preset_bindings = vec![
            PresetBindingRule {
                id: "rule_peaceful".to_string(),
                target_type: PresetBindingType::Album,
                target_key: "Peaceful Days".to_string(),
                target_label: "Album: Peaceful Days".to_string(),
                preset_name: "Warm Studio Master".to_string(),
            },
        ];

        // 1. Calling apply_startup_preset on startup
        app.apply_startup_preset();
        assert_eq!(app.last_applied_auto_preset.as_deref(), Some("Warm Studio Master"), "Startup preset must be immediately loaded");
        assert!(app.preset_status_notification.is_none(), "Startup preset loading must be silent (no toast popup)");

        // 2. Track changes to track 2 in SAME album
        let track2 = crate::models::Track {
            title: "Acoustic Evening".to_string(),
            artist: "Guitarist".to_string(),
            album: "Peaceful Days".to_string(),
            genre: Some("Acoustic".to_string()),
            ..Default::default()
        };
        app.check_and_apply_auto_binding(&track2);
        assert!(app.preset_status_notification.is_none(), "Same preset across songs must not spam any toast notifications");
    }
}
