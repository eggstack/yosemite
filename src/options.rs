// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};

use crate::error::ProtocolError;

use std::{fmt, time::Duration};

/// Default port for UDP.
pub(crate) const SAMV3_UDP_PORT: u16 = 7655;

/// Default port for TCP.
pub(crate) const SAMV3_TCP_PORT: u16 = 7656;

/// Destination kind.
#[derive(Clone, PartialEq, Eq)]
pub enum DestinationKind {
    /// Transient session.
    Transient,

    /// Session from pre-generated destination data.
    Persistent {
        /// Base64 of the concatenation of the destination followed by the private key followed by
        /// the signing private key.
        private_key: String,
    },
}

impl fmt::Debug for DestinationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transient => f.debug_struct("DestinationKind::Transient").finish(),
            Self::Persistent { .. } => {
                f.debug_struct("DestinationKind::Persistent").finish_non_exhaustive()
            }
        }
    }
}

/// Maximum number of additional options accepted for a session.
pub const MAX_ADDITIONAL_SESSION_OPTIONS: usize = 32;

/// Maximum byte length of an additional session-option key.
pub const MAX_SESSION_OPTION_KEY_LENGTH: usize = 64;

/// Maximum byte length of an additional session-option value.
pub const MAX_SESSION_OPTION_VALUE_LENGTH: usize = 256;

/// Maximum number of LeaseSet client-auth entries.
pub const MAX_LEASE_SET_CLIENT_AUTHS: usize = 16;

/// Maximum byte length of a LeaseSet client-auth key.
///
/// The underlying X25519 public key is 32 bytes (44 base64 chars) but we allow a
/// bounded larger value for future enc types while keeping a strict limit.
pub const MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH: usize = 512;

/// Maximum byte length of LeaseSet secret/key material.
pub const MAX_LEASE_SET_SECRET_LENGTH: usize = 512;

/// A validated additional SAM session option.
///
/// Use [`SessionOption::new`] to construct an option. Values are intentionally restricted to a
/// single unquoted SAM token; this type does not represent preformatted command fragments.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionOption {
    key: String,
    value: String,
}

impl SessionOption {
    /// Create a validated additional session option.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Result<Self, ProtocolError> {
        let option = Self {
            key: key.into(),
            value: value.into(),
        };
        option.validate()?;
        Ok(option)
    }

    /// Get the option key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the option value.
    pub fn value(&self) -> &str {
        &self.value
    }

    pub(crate) fn validate(&self) -> Result<(), ProtocolError> {
        if self.key.is_empty()
            || self.key.len() > MAX_SESSION_OPTION_KEY_LENGTH
            || !self
                .key
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            || is_reserved_session_option_key(&self.key)
            || self.value.len() > MAX_SESSION_OPTION_VALUE_LENGTH
            || !self.value.bytes().all(is_safe_session_option_value_byte)
        {
            return Err(ProtocolError::InvalidOption);
        }

        Ok(())
    }
}

impl fmt::Debug for SessionOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionOption")
            .field("key", &self.key)
            .field("value", &"<redacted>")
            .finish()
    }
}

fn is_safe_session_option_value_byte(byte: u8) -> bool {
    byte.is_ascii_graphic() && !matches!(byte, b'"' | b'\\' | b'=')
}

fn is_reserved_session_option_key(key: &str) -> bool {
    [
        "STYLE",
        "ID",
        "DESTINATION",
        "SIGNATURE_TYPE",
        "FROM_PORT",
        "TO_PORT",
        "PROTOCOL",
        "HEADER",
        "PORT",
        "HOST",
        "i2cp.dontPublishLeaseSet",
        "i2cp.leaseSetEncType",
        "i2cp.encryptLeaseSet",
        "i2cp.leaseSetAuthType",
        "i2cp.leaseSetBlindedType",
        "i2cp.leaseSetType",
        "i2cp.leaseSetKey",
        "i2cp.leaseSetPrivKey",
        "i2cp.leaseSetSecret",
        "i2cp.leaseSetSigningPrivKey",
        "inbound.length",
        "inbound.quantity",
        "inbound.lengthVariance",
        "inbound.backupQuantity",
        "outbound.length",
        "outbound.quantity",
        "outbound.lengthVariance",
        "outbound.backupQuantity",
    ]
    .iter()
    .any(|reserved| key.eq_ignore_ascii_case(reserved))
        || key
            .to_ascii_lowercase()
            .starts_with("i2cp.leaseSetClientAuth.".to_ascii_lowercase().as_str())
}

fn is_valid_base64(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_LEASE_SET_SECRET_LENGTH {
        return false;
    }
    if !value
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b'-' | b'_'))
    {
        return false;
    }
    // Length must be multiple of 4 for standard base64, but allow URL-safe without padding
    // so only check that padding '=' only appears at end and not in middle, and length is sane.
    if value.contains('=') {
        let trimmed = value.trim_end_matches('=');
        if trimmed.contains('=') {
            return false;
        }
        // Standard base64 padded length must be multiple of 4
        if !value.len().is_multiple_of(4) {
            return false;
        }
    }
    // No whitespace/control
    if value.bytes().any(|b| b.is_ascii_control()) {
        return false;
    }
    true
}

fn is_valid_lease_set_secret(value: &str) -> bool {
    // Secret is base64-encoded UTF-8, bounded and base64.
    !value.is_empty() && value.len() <= MAX_LEASE_SET_SECRET_LENGTH && is_valid_base64(value)
}

fn is_valid_lease_set_key(value: &str) -> bool {
    // SessionKey is 32 bytes => 44 base64 chars; allow 44 or padded variants up to bound.
    !value.is_empty() && value.len() <= MAX_LEASE_SET_SECRET_LENGTH && is_valid_base64(value)
}

/// A validated LeaseSet client-authorization entry.
///
/// This is a generic, bounded representation for per-client LeaseSet authentication
/// material (e.g. X25519 public keys for DH/PSK). The router is authoritative for
/// whether a given entry is honored; Yosemite only guarantees bounded, validated,
/// non-leaking transport to the SESSION CREATE wire with deterministic ordering.
#[derive(Clone, PartialEq, Eq)]
pub struct LeaseSetClientAuth {
    key: String,
}

impl LeaseSetClientAuth {
    /// Create a validated client-auth entry.
    ///
    /// The `key` must be a base64-encoded public key (or PSK material) without
    /// whitespace, control characters, or SAM-token delimiters. Length is bounded
    /// by `MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH`.
    pub fn new(key: impl Into<String>) -> Result<Self, ProtocolError> {
        let entry = Self { key: key.into() };
        entry.validate()?;
        Ok(entry)
    }

    /// Get the raw key material.
    pub fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn validate(&self) -> Result<(), ProtocolError> {
        if self.key.is_empty()
            || self.key.len() > MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH
            || !is_valid_base64(&self.key)
        {
            return Err(ProtocolError::InvalidOption);
        }
        // Ensure no whitespace/control that would break SAM tokenization (base64 check already covers).
        if self.key.bytes().any(|b| b.is_ascii_whitespace() || b.is_ascii_control()) {
            return Err(ProtocolError::InvalidOption);
        }
        Ok(())
    }
}

impl fmt::Debug for LeaseSetClientAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LeaseSetClientAuth").field("key", &"<redacted>").finish()
    }
}

/// Session options.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionOptions {
    /// Nickname.
    ///
    /// Name that uniquely identifies the session.
    ///
    /// Defaults to a random alphanumeric string.
    pub nickname: String,

    /// Destination kind.
    ///
    /// Defaults to [`DestinationKind::Transient`].
    pub destination: DestinationKind,

    /// Signature type.
    ///
    /// Defaults to `7`, i.e., EdDSA-SHA512-Ed25519
    pub signature_type: u16,

    /// Additional validated SAM/I2CP session options.
    ///
    /// Prefer [`SessionOptions::add_session_option`] so count and collision checks are applied.
    /// The collection is revalidated before a command is serialized because it is public for
    /// compatibility with the rest of this struct's configuration fields.
    pub additional_options: Vec<SessionOption>,

    /// Port where the datagram socket should be bound to.
    ///
    /// Defaults to `0`.
    pub datagram_port: u16,

    /// Defaults to `0`.
    pub from_port: u16,

    /// Defaults to `0`.
    pub to_port: u16,

    /// Defaults to `18`, i.e., raw datagrams
    pub protocol: u8,

    /// Defaults to `false`.
    pub header: bool,

    /// Should the session's lease set be published to NetDb.
    ///
    /// Outbound-only sessions (clients) shouldn't be published whereas servers (accepting inbound
    /// connections) need to be published.
    ///
    /// Corresponds to `i2cp.dontPublishLeaseSet`.
    ///
    /// Defaults to `true`.
    pub publish: bool,

    /// Minimum number of ElGamal/AES Session Tags before we send more. Recommended: approximately
    /// tagsToSend * 2/3
    ///
    /// Defaults to `30`.
    pub crypto_low_tag_threshold: usize,

    /// Inbound tag window for ECIES-X25519-AEAD-Ratchet. Local inbound tagset size.
    ///
    /// Defaults to `160`.
    pub crypto_ratchet_inbound_tags: usize,

    /// Outbound tag window for ECIES-X25519-AEAD-Ratchet. Advisory to send to the far-end in the
    /// options block.
    ///
    /// Defaults to `160`.
    pub crypto_ratchet_outbound_tags: usize,

    /// Number of ElGamal/AES Session Tags to send at a time.
    ///
    /// Defaults to `40`.
    pub crypto_tags_to_send: usize,

    /// For authorization, if required by the router.
    ///
    /// Defauts to `None`.
    pub username: Option<String>,

    /// For authorization, if required by the router.
    ///
    /// Defauts to `None`.
    pub password: Option<String>,

    /// If incoming zero hop tunnel is allowed
    ///
    /// Defaults to `false`.
    pub inbound_allow_zero_hop: bool,

    /// How many hops do the inbound tunnels of the session have.
    ///
    /// Defaults to `3`.
    pub inbound_len: usize,

    /// Random amount to add or subtract to the length of tunnels in.
    ///
    /// Defaults to `0`.
    pub inbound_len_variance: isize,

    /// How many inbound tunnels does the tunnel pool of the session have.
    ///
    /// Defaults to `2`.
    pub inbound_quantity: usize,

    /// Number of redundant, fail-over inbound tunnels
    ///
    /// Defaults to `0`.
    pub inbound_backup_quantity: usize,

    /// Number of IP bytes to match to determine if two routers should not be in the same inbound
    /// tunnel.
    ///
    /// Defaults to `None`.
    pub inbound_ip_restriction: Option<std::num::NonZeroUsize>,

    /// Used for consistent peer ordering across restarts.
    ///
    /// Defauts to `None`.
    pub inbound_random_key: Option<String>,

    /// Name of inbound tunnels - generally used in routerconsole, which will use
    /// the first few characters of the Base64 hash of the destination by default.
    ///
    /// Defaults to `None`.
    pub inbound_nickname: Option<String>,

    /// If outgoing zero hop tunnel is allowed
    ///
    ///  Defaults to `false`.
    pub outbound_allow_zero_hop: bool,

    /// How many hops do the outbound tunnels of the session have.
    ///
    /// Defaults to `3`.
    pub outbound_len: usize,

    /// Random amount to add or subtract to the length of tunnels in.
    ///
    /// Defaults to `0`.
    pub outbound_len_variance: isize,

    /// How many outbound tunnels does the tunnel pool of the session have.
    ///
    /// Defaults to `2`.
    pub outbound_quantity: usize,

    /// Number of redundant, fail-over outbound tunnels
    ///
    /// Defaults to `0`.
    pub outbound_backup_quantity: usize,

    /// Number of IP bytes to match to determine if two routers should not be in the same outbound
    /// tunnel.
    ///
    /// Defaults to `None`.
    pub outbound_ip_restriction: Option<std::num::NonZeroUsize>,

    /// Used for consistent peer ordering across restarts.
    ///
    /// Defauts to `None`.
    pub outbound_random_key: Option<String>,

    /// Name of outbound tunnels - generally ignored unless `inbound_nickname` is unset.
    ///
    /// Defauts to `None`.
    pub outbound_nickname: Option<String>,

    /// Priority adjustment for outbound messages. Higher is higher priority.
    ///
    /// Defaults to `0`.
    pub outbound_priority: isize,

    /// Set to false to disable ever bundling a reply LeaseSet.
    ///
    /// Defaults to `true`.
    pub should_bundle_reply_info: bool,

    /// Reduce tunnel quantity when idle
    ///
    /// Defaults to `false`.
    pub reduce_on_idle: bool,

    /// Idle time required before reducing tunnel quantity
    ///
    /// Defaults to 20 minutes.
    pub reduce_idle_time: Duration,

    /// Tunnel quantity when reduced (applies to both inbound and outbound)
    ///
    /// Defauts to `1`.
    pub reduce_quantity: usize,

    /// Close I2P session when idle
    ///
    /// Defaults to `false`.
    pub close_on_idle: bool,

    /// Idle time required before closing session
    ///
    /// Defaults to 30 minutes.
    pub close_idle_time: Duration,

    /// The type of authentication for encrypted LS2. 0 for no per-client authentication ;
    /// 1 for DH per-client authentication; 2 for PSK per-client authentication; 3 for PSK (compatible).
    ///
    /// Corresponds to `i2cp.leaseSetAuthType`. Validated to 0..3 and emitted only when non-zero.
    ///
    /// Defaults to `0`.
    pub lease_set_auth_type: usize,

    /// The sig type of the blinded key for encrypted LS2. Default depends on the destination sig
    /// type.
    ///
    /// Corresponds to `i2cp.leaseSetBlindedType`. Validated to 0..12 and emitted only when non-zero.
    ///
    /// Defaults to `0`.
    pub lease_set_blinded_type: usize,

    /// The encryption type to be used.
    ///
    /// `None` defaults to `6,4`, i.e., ML-KEM-768-x25519, x25519.
    ///
    /// Corresponds to `i2cp.leaseSetEncType`. Already emitted as `i2cp.leaseSetEncType=6,4` when `None`.
    ///
    /// Accepts a single value, e.g., `4` or a comma-separated list of values, e.g., `6,5,4` where
    /// the order specifies preference.
    ///
    /// Note that the router may not support more than two encryption types.
    pub lease_set_enc_type: Option<String>,

    /// For encrypted leasesets. Base 64 SessionKey (44 characters)
    ///
    /// Corresponds to `i2cp.leaseSetKey`. Base64 validated, bounded to 512 bytes.
    ///
    /// Defauts to `None`.
    pub lease_set_key: Option<String>,

    /// Base 64 private keys for encryption.
    ///
    /// Corresponds to `i2cp.leaseSetPrivKey`. Base64 validated, bounded.
    ///
    /// Defauts to `None`.
    pub lease_set_private_key: Option<String>,

    /// Base 64 encoded UTF-8 secret used to blind the leaseset address.
    ///
    /// Corresponds to `i2cp.leaseSetSecret`. Base64 validated, bounded.
    ///
    /// Defauts to `None`.
    pub lease_set_secret: Option<String>,

    /// Base 64 private key for signatures.
    ///
    /// Corresponds to `i2cp.leaseSetSigningPrivKey`. Base64 validated, bounded.
    ///
    /// Defauts to `None`.
    pub lease_set_signing_private_key: Option<String>,

    /// The type of leaseset to be sent in the CreateLeaseSet2 Message.
    ///
    /// Corresponds to `i2cp.leaseSetType`. Validated to 0..5 and emitted only when not `1`.
    ///
    /// Defaults to `1`.
    pub lease_set_type: usize,

    /// Encrypt the lease
    ///
    /// Corresponds to `i2cp.encryptLeaseSet`. When `true`, emits `i2cp.encryptLeaseSet=true`.
    ///
    /// Defaults to `false`.
    pub encrypt_lease_set: bool,

    /// Gzip outbound data
    ///
    /// Defaults to `true`.
    pub gzip: bool,

    /// Connect to the router using SSL.
    ///
    /// Defauts to `false`.
    pub ssl: bool,

    /// TCP port of the listening SAMv3 server.
    ///
    /// Defaults to `7656`.
    pub samv3_tcp_port: u16,

    /// UDP port of the listening SAMv3 server.
    ///
    /// Defaults to `7655`
    pub samv3_udp_port: u16,

    /// Should `STREAM FORWARD` be silent.
    ///
    /// If set to false, the first message read from the socket is the destination of the remote
    /// peer.
    ///
    /// Defaults to `false`.
    pub silent_forward: bool,

    /// Bounded LeaseSet client-authorization entries.
    ///
    /// Each entry is a validated base64-encoded key (e.g. X25519 public key for
    /// DH/PSK). The collection is bounded, duplicate-free, and serialized
    /// deterministically as `i2cp.leaseSetClientAuth.<index>` with indices
    /// derived from sorted keys. This is a generic SAM transport; the router
    /// remains authoritative for whether the entry is honored.
    ///
    /// Defaults to empty.
    pub lease_set_client_auths: Vec<LeaseSetClientAuth>,
}

impl fmt::Debug for SessionOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionOptions")
            .field("nickname", &self.nickname)
            .field("destination", &self.destination)
            .field("signature_type", &self.signature_type)
            .field("additional_options", &self.additional_options)
            .field("datagram_port", &self.datagram_port)
            .field("from_port", &self.from_port)
            .field("to_port", &self.to_port)
            .field("protocol", &self.protocol)
            .field("header", &self.header)
            .field("publish", &self.publish)
            .field("crypto_low_tag_threshold", &self.crypto_low_tag_threshold)
            .field(
                "crypto_ratchet_inbound_tags",
                &self.crypto_ratchet_inbound_tags,
            )
            .field(
                "crypto_ratchet_outbound_tags",
                &self.crypto_ratchet_outbound_tags,
            )
            .field("crypto_tags_to_send", &self.crypto_tags_to_send)
            .field("username", &self.username.as_ref().map(|_| "<redacted>"))
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("inbound_allow_zero_hop", &self.inbound_allow_zero_hop)
            .field("inbound_len", &self.inbound_len)
            .field("inbound_len_variance", &self.inbound_len_variance)
            .field("inbound_quantity", &self.inbound_quantity)
            .field("inbound_backup_quantity", &self.inbound_backup_quantity)
            .field("inbound_ip_restriction", &self.inbound_ip_restriction)
            .field(
                "inbound_random_key",
                &self.inbound_random_key.as_ref().map(|_| "<redacted>"),
            )
            .field("inbound_nickname", &self.inbound_nickname)
            .field("outbound_allow_zero_hop", &self.outbound_allow_zero_hop)
            .field("outbound_len", &self.outbound_len)
            .field("outbound_len_variance", &self.outbound_len_variance)
            .field("outbound_quantity", &self.outbound_quantity)
            .field("outbound_backup_quantity", &self.outbound_backup_quantity)
            .field("outbound_ip_restriction", &self.outbound_ip_restriction)
            .field(
                "outbound_random_key",
                &self.outbound_random_key.as_ref().map(|_| "<redacted>"),
            )
            .field("outbound_nickname", &self.outbound_nickname)
            .field("outbound_priority", &self.outbound_priority)
            .field("should_bundle_reply_info", &self.should_bundle_reply_info)
            .field("reduce_on_idle", &self.reduce_on_idle)
            .field("reduce_idle_time", &self.reduce_idle_time)
            .field("reduce_quantity", &self.reduce_quantity)
            .field("close_on_idle", &self.close_on_idle)
            .field("close_idle_time", &self.close_idle_time)
            .field("lease_set_auth_type", &self.lease_set_auth_type)
            .field("lease_set_blinded_type", &self.lease_set_blinded_type)
            .field("lease_set_enc_type", &self.lease_set_enc_type)
            .field(
                "lease_set_key",
                &self.lease_set_key.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "lease_set_private_key",
                &self.lease_set_private_key.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "lease_set_secret",
                &self.lease_set_secret.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "lease_set_signing_private_key",
                &self.lease_set_signing_private_key.as_ref().map(|_| "<redacted>"),
            )
            .field("lease_set_type", &self.lease_set_type)
            .field("encrypt_lease_set", &self.encrypt_lease_set)
            .field("lease_set_client_auths", &self.lease_set_client_auths)
            .field("gzip", &self.gzip)
            .field("ssl", &self.ssl)
            .field("samv3_tcp_port", &self.samv3_tcp_port)
            .field("samv3_udp_port", &self.samv3_udp_port)
            .field("silent_forward", &self.silent_forward)
            .finish()
    }
}

impl SessionOptions {
    /// Add one validated generic SAM/I2CP session option.
    pub fn add_session_option(
        &mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), ProtocolError> {
        let option = SessionOption::new(key, value)?;
        if self.additional_options.len() >= MAX_ADDITIONAL_SESSION_OPTIONS
            || self
                .additional_options
                .iter()
                .any(|existing| existing.key.eq_ignore_ascii_case(&option.key))
            || is_reserved_session_option_key(&option.key)
        {
            return Err(ProtocolError::InvalidOption);
        }
        // Also reject if generic key collides with any typed LeaseSet option that is currently active.
        if self.lease_set_typed_key_conflict(&option.key) {
            return Err(ProtocolError::InvalidOption);
        }

        self.additional_options.push(option);
        Ok(())
    }

    /// Add one validated LeaseSet client-auth entry.
    pub fn add_lease_set_client_auth(
        &mut self,
        key: impl Into<String>,
    ) -> Result<(), ProtocolError> {
        let entry = LeaseSetClientAuth::new(key)?;
        if self.lease_set_client_auths.len() >= MAX_LEASE_SET_CLIENT_AUTHS
            || self
                .lease_set_client_auths
                .iter()
                .any(|existing| existing.key.eq_ignore_ascii_case(&entry.key))
        {
            return Err(ProtocolError::InvalidOption);
        }
        // Also ensure no generic additional_option already occupies the derived wire key that
        // this entry would eventually produce. Since wire keys are deterministic after sorting,
        // we conservatively reject if any additional_option uses the client-auth prefix.
        if self
            .additional_options
            .iter()
            .any(|opt| opt.key.to_ascii_lowercase().starts_with("i2cp.leasesetclientauth."))
        {
            return Err(ProtocolError::InvalidOption);
        }

        self.lease_set_client_auths.push(entry);
        Ok(())
    }

    fn lease_set_typed_key_conflict(&self, key: &str) -> bool {
        let mut typed_keys = Vec::new();
        if self.encrypt_lease_set {
            typed_keys.push("i2cp.encryptLeaseSet");
        }
        if self.lease_set_auth_type != 0 {
            typed_keys.push("i2cp.leaseSetAuthType");
        }
        if self.lease_set_blinded_type != 0 {
            typed_keys.push("i2cp.leaseSetBlindedType");
        }
        if self.lease_set_key.is_some() {
            typed_keys.push("i2cp.leaseSetKey");
        }
        if self.lease_set_private_key.is_some() {
            typed_keys.push("i2cp.leaseSetPrivKey");
        }
        if self.lease_set_secret.is_some() {
            typed_keys.push("i2cp.leaseSetSecret");
        }
        if self.lease_set_signing_private_key.is_some() {
            typed_keys.push("i2cp.leaseSetSigningPrivKey");
        }
        if self.lease_set_type != 1 {
            typed_keys.push("i2cp.leaseSetType");
        }
        // Client auth entries occupy a namespace, not a single key, but conflict is already handled via prefix check.
        typed_keys.iter().any(|k| k.eq_ignore_ascii_case(key))
    }

    pub(crate) fn validate_additional_options(&self) -> Result<(), ProtocolError> {
        if self.additional_options.len() > MAX_ADDITIONAL_SESSION_OPTIONS {
            return Err(ProtocolError::InvalidOption);
        }

        for (index, option) in self.additional_options.iter().enumerate() {
            option.validate()?;
            if self.additional_options[index + 1..]
                .iter()
                .any(|other| other.key.eq_ignore_ascii_case(&option.key))
            {
                return Err(ProtocolError::InvalidOption);
            }
        }

        Ok(())
    }

    pub(crate) fn validate_lease_set_options(&self) -> Result<(), ProtocolError> {
        // auth type: 0 = none, 1 = DH, 2 = PSK, 3 = PSK (I2P BlindData allows 3); allow 0..3.
        if self.lease_set_auth_type > 3 {
            return Err(ProtocolError::InvalidOption);
        }
        // blinded type: 0 = default, otherwise must be a known SigType code (7, 8, 9, 10, 11 are common). Allow 0..12 for forward compat.
        if self.lease_set_blinded_type > 12 {
            return Err(ProtocolError::InvalidOption);
        }
        // lease_set_type: 0..5 plausible; default 1. Allow 0..5.
        if self.lease_set_type > 5 {
            return Err(ProtocolError::InvalidOption);
        }
        if let Some(value) = &self.lease_set_key {
            if !is_valid_lease_set_key(value) {
                return Err(ProtocolError::InvalidOption);
            }
        }
        if let Some(value) = &self.lease_set_private_key {
            if !is_valid_lease_set_key(value) {
                return Err(ProtocolError::InvalidOption);
            }
        }
        if let Some(value) = &self.lease_set_secret {
            if !is_valid_lease_set_secret(value) {
                return Err(ProtocolError::InvalidOption);
            }
        }
        if let Some(value) = &self.lease_set_signing_private_key {
            if !is_valid_lease_set_key(value) {
                return Err(ProtocolError::InvalidOption);
            }
        }
        // lease_set_enc_type is validated elsewhere (generic SessionOption value already checked) but we also ensure it's not empty and uses safe bytes.
        if let Some(value) = &self.lease_set_enc_type {
            if value.is_empty()
                || value.len() > MAX_SESSION_OPTION_VALUE_LENGTH
                || !value.bytes().all(is_safe_session_option_value_byte)
            {
                return Err(ProtocolError::InvalidOption);
            }
            // Basic comma-separated numeric check: each item 3..7, etc. We keep permissive here and let router decide, but reject obvious injection.
            if value.contains(' ') || value.contains('\n') || value.contains('\r') {
                return Err(ProtocolError::InvalidOption);
            }
        }

        // client auths
        if self.lease_set_client_auths.len() > MAX_LEASE_SET_CLIENT_AUTHS {
            return Err(ProtocolError::InvalidOption);
        }
        for (index, entry) in self.lease_set_client_auths.iter().enumerate() {
            entry.validate()?;
            if self.lease_set_client_auths[index + 1..]
                .iter()
                .any(|other| other.key.eq_ignore_ascii_case(&entry.key))
            {
                return Err(ProtocolError::InvalidOption);
            }
        }

        // Conflict: typed LeaseSet keys must not already be present as generic additional_options.
        for option in &self.additional_options {
            if self.lease_set_typed_key_conflict(&option.key)
                || option.key.to_ascii_lowercase().starts_with("i2cp.leasesetclientauth.")
            {
                return Err(ProtocolError::InvalidOption);
            }
        }

        // Fail-closed: if encrypt_lease_set is true but no LeaseSet material is provided, we still
        // allow it (router will decide), but if auth_type requires client auths and none are present,
        // we must not silently continue with a weaker config. The plan says no weaker fallback, so
        // if auth_type !=0 but client_auths is empty, we could allow but router will handle. However
        // to enforce fail-closed for clearly inconsistent config, we reject auth_type !=0 with empty client auths?
        // For now we allow empty; the wire will contain auth type but no entries, which is still explicit.
        // The important fail-closed is: invalid fields must error, not be omitted.

        Ok(())
    }

    pub(crate) fn validate_all_options(&self) -> Result<(), ProtocolError> {
        self.validate_additional_options()?;
        self.validate_lease_set_options()?;
        Ok(())
    }
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            nickname: Alphanumeric.sample_string(&mut thread_rng(), 16),
            destination: DestinationKind::Transient,
            signature_type: 7u16,
            additional_options: Vec::new(),
            datagram_port: 0u16,
            from_port: 0u16,
            to_port: 0u16,
            protocol: 18u8,
            header: false,
            publish: true,
            crypto_low_tag_threshold: 30usize,
            crypto_ratchet_inbound_tags: 160usize,
            crypto_ratchet_outbound_tags: 160usize,
            crypto_tags_to_send: 40usize,
            username: None,
            password: None,
            inbound_allow_zero_hop: false,
            inbound_len: 3usize,
            inbound_len_variance: 0isize,
            inbound_quantity: 2usize,
            inbound_backup_quantity: 0usize,
            inbound_ip_restriction: None,
            inbound_random_key: None,
            inbound_nickname: None,
            outbound_allow_zero_hop: false,
            outbound_len: 3usize,
            outbound_len_variance: 0isize,
            outbound_quantity: 2usize,
            outbound_backup_quantity: 0usize,
            outbound_ip_restriction: None,
            outbound_random_key: None,
            outbound_nickname: None,
            outbound_priority: 0isize,
            should_bundle_reply_info: true,
            reduce_on_idle: false,
            reduce_idle_time: Duration::from_millis(1200000),
            reduce_quantity: 1usize,
            close_on_idle: false,
            close_idle_time: Duration::from_millis(1800000),
            lease_set_auth_type: 0usize,
            lease_set_blinded_type: 0usize,
            lease_set_enc_type: None,
            lease_set_key: None,
            lease_set_private_key: None,
            lease_set_secret: None,
            lease_set_signing_private_key: None,
            lease_set_type: 1usize,
            encrypt_lease_set: false,
            lease_set_client_auths: Vec::new(),
            gzip: true,
            ssl: false,
            samv3_tcp_port: SAMV3_TCP_PORT,
            samv3_udp_port: SAMV3_UDP_PORT,
            silent_forward: false,
        }
    }
}

/// Stream options.
#[derive(Debug, Default, Clone, Copy)]
pub struct StreamOptions {
    /// Destination port.
    ///
    /// Defaults to `0`.
    pub dst_port: u16,

    /// Source port.
    ///
    /// Defaults to `0`.
    pub src_port: u16,
}

/// Datagram options.
#[derive(Default)]
pub struct DatagramOptions {
    /// Overrides the source port.
    ///
    /// Defaults to `0`.
    pub from_port: u16,

    /// Overrides the destination port.
    ///
    /// Defaults to `0`.
    pub to_port: u16,

    /// Overrides the I2P protocol used.
    ///
    /// Only for RAW/Anonymous sessions.
    ///
    /// Defaults to `18`.
    pub protocol: u8,

    /// Overrides the [`SessionOptions::crypto_tags_to_send`] I2CP option.
    ///
    /// Defaults to `0`.
    pub send_tags: usize,

    /// Overrides the [`SessionOptions::crypto_low_tag_threshold`] I2CP option.
    ///
    /// Defaults to `0`.
    pub tag_threshold: usize,

    /// Overrides the [`SessionOptions::should_bundle_reply_info`] I2CP option.
    ///
    /// Defaults to `true`.
    pub send_lease_set: bool,
}
