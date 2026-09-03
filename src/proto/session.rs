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

use crate::{
    error::ProtocolError,
    options::{SessionOptions, StreamOptions},
    proto::parser::Response,
    style::private::SessionParameters,
    DestinationKind,
};

/// Logging target for the file.
const LOG_TARGET: &str = "yosemite::proto::session";

/// Stream kind
#[derive(Debug, PartialEq, Eq, Clone)]
enum StreamKind {
    /// `STREAM ACCEPT` has been sent.
    Accept,

    /// `STREAM CONNECT` has been sent.
    Connect,

    /// `STREAM FORWARD` has been sent.
    Forward,
}

/// Virtual stream state.
#[derive(Debug, PartialEq, Eq, Clone)]
enum StreamState {
    /// Stream state is uninitialized.
    Uninitialized,

    /// Stream is being handshaked.
    Handshaking,

    /// Stream has been handshaked.
    Handshaked,

    /// `STREAM CONNECT`/`STREAM ACCEPT` is pending.
    Pending(StreamKind),
}

/// Session state.
#[derive(Debug, PartialEq, Eq, Clone)]
enum SessionState {
    /// Session is uninitialized.
    Uninitialized,

    /// Handshake has been sent to router.
    Handshaking,

    /// Session has been handshaked.
    Handshaked,

    /// `SESSION CREATE` message has been sent.
    SessionCreatePending,

    /// `SESSION ADD` message has been sent.
    SubsessionCreatePending {
        /// Created destination.
        destination: String,
    },

    /// Session is active.
    Active {
        /// Created destination.
        destination: String,

        /// Stream state
        stream_state: StreamState,
    },

    /// Session state has been poisoned.
    Poisoned,
}

/// State machine for SAMv3 virtual streams.
#[derive(Clone)]
pub struct SessionController {
    /// Session options.
    options: SessionOptions,

    /// Session state.
    state: SessionState,
}

impl SessionController {
    /// Create new [`SessionController`] from `options`.
    pub fn new(options: SessionOptions) -> Result<Self, ProtocolError> {
        options.validate_all_options()?;

        Ok(Self {
            options,
            state: SessionState::Uninitialized,
        })
    }

    /// Create new [`SessionController`] for a subsession from primary session's state.
    pub(crate) fn new_for_subsession(&self, options: SessionOptions) -> Self {
        match &self.state {
            SessionState::Active {
                destination,
                stream_state: StreamState::Uninitialized,
            } => Self {
                options,
                state: SessionState::Active {
                    destination: destination.clone(),
                    stream_state: StreamState::Uninitialized,
                },
            },
            _ => unreachable!(),
        }
    }

    /// Initialize new session by handshaking with the router.
    pub fn handshake_session(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Uninitialized => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "send handshake for session",
                );
                self.state = SessionState::Handshaking;

                Ok(String::from("HELLO VERSION\n").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Create new session with either transient or persistent destination.
    pub fn create_session(
        &mut self,
        parameters: SessionParameters,
    ) -> Result<Vec<u8>, ProtocolError> {
        // SessionOptions is publicly mutable, so validate again immediately before constructing
        // any wire command. This also keeps invalid options from changing the controller state.
        self.options.validate_all_options()?;

        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaked => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    destination = ?self.options.destination,
                    "create new session",
                );
                self.state = SessionState::SessionCreatePending;

                let mut command = format!(
                    "SESSION CREATE STYLE={} ID={} ",
                    parameters.style, self.options.nickname
                );

                for (key, value) in parameters.options {
                    command += format!("{key}={value} ").as_str();
                }

                match &self.options.destination {
                    DestinationKind::Transient => {
                        command += "DESTINATION=TRANSIENT ";
                    }
                    DestinationKind::Persistent { private_key } => {
                        command += format!("DESTINATION={private_key} ").as_str();
                    }
                }

                match parameters.style.as_str() {
                    "PRIMARY" => {}
                    "STREAM" => {}
                    "DATAGRAM" => {
                        command += format!(
                            "FROM_PORT={} TO_PORT={} ",
                            self.options.from_port, self.options.to_port,
                        )
                        .as_str();
                    }
                    "DATAGRAM2" => {}
                    "DATAGRAM3" => {}
                    "RAW" => {
                        command += format!(
                            "FROM_PORT={} TO_PORT={} PROTOCOL={} HEADER={} ",
                            self.options.from_port,
                            self.options.to_port,
                            self.options.protocol,
                            self.options.header,
                        )
                        .as_str();
                    }
                    _ => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            style = %parameters.style,
                            "cannot create session, non-supported session style",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                }

                if !self.options.publish {
                    command += "i2cp.dontPublishLeaseSet=true ";
                }

                match &self.options.lease_set_enc_type {
                    None => {
                        command += "i2cp.leaseSetEncType=6,4 ";
                    }
                    Some(value) => {
                        command += format!("i2cp.leaseSetEncType={value} ").as_str();
                    }
                }

                command += format!(
                    "inbound.length={} inbound.quantity={} ",
                    self.options.inbound_len, self.options.inbound_quantity
                )
                .as_str();

                command += format!(
                    "outbound.length={} outbound.quantity={} ",
                    self.options.outbound_len, self.options.outbound_quantity
                )
                .as_str();

                command += format!(
                    "inbound.lengthVariance={} inbound.backupQuantity={} ",
                    self.options.inbound_len_variance, self.options.inbound_backup_quantity
                )
                .as_str();

                command += format!(
                    "outbound.lengthVariance={} outbound.backupQuantity={} ",
                    self.options.outbound_len_variance, self.options.outbound_backup_quantity
                )
                .as_str();

                command += format!("SIGNATURE_TYPE={}", self.options.signature_type).as_str();

                // LeaseSet typed options — only emitted when non-default/Some, preserving default wire.
                // Validation has already succeeded via validate_all_options(), so we can emit directly
                // without re-validating. Order is deterministic via sorted keys where applicable.
                let mut lease_options: Vec<(String, String)> = Vec::new();
                if self.options.encrypt_lease_set {
                    lease_options.push(("i2cp.encryptLeaseSet".to_string(), "true".to_string()));
                }
                if self.options.lease_set_auth_type != 0 {
                    lease_options.push((
                        "i2cp.leaseSetAuthType".to_string(),
                        self.options.lease_set_auth_type.to_string(),
                    ));
                }
                if self.options.lease_set_blinded_type != 0 {
                    lease_options.push((
                        "i2cp.leaseSetBlindedType".to_string(),
                        self.options.lease_set_blinded_type.to_string(),
                    ));
                }
                if self.options.lease_set_type != 1 {
                    lease_options.push((
                        "i2cp.leaseSetType".to_string(),
                        self.options.lease_set_type.to_string(),
                    ));
                }
                if let Some(value) = &self.options.lease_set_key {
                    lease_options.push(("i2cp.leaseSetKey".to_string(), value.clone()));
                }
                if let Some(value) = &self.options.lease_set_private_key {
                    lease_options.push(("i2cp.leaseSetPrivateKey".to_string(), value.clone()));
                }
                if let Some(value) = &self.options.lease_set_secret {
                    lease_options.push(("i2cp.leaseSetSecret".to_string(), value.clone()));
                }
                if let Some(value) = &self.options.lease_set_signing_private_key {
                    lease_options
                        .push(("i2cp.leaseSetSigningPrivateKey".to_string(), value.clone()));
                }

                // Number each authorization mode independently, in deterministic wire-value
                // order. Validation has already rejected duplicate names within a mode.
                for mode in [
                    crate::options::LeaseSetClientAuthMode::Dh,
                    crate::options::LeaseSetClientAuthMode::Psk,
                ] {
                    let mut auths = self
                        .options
                        .lease_set_client_auths
                        .iter()
                        .filter(|auth| auth.mode() == mode)
                        .collect::<Vec<_>>();
                    auths.sort_by_key(|auth| auth.wire_value());
                    for (index, auth) in auths.iter().enumerate() {
                        lease_options.push((
                            format!("{}{index}", auth.wire_key_prefix()),
                            auth.wire_value(),
                        ));
                    }
                }

                // Sort typed options by their complete canonical key. The per-mode numbering is
                // assigned above and is therefore unaffected by insertion order.
                lease_options.sort_by(|a, b| a.0.cmp(&b.0));
                for (key, value) in lease_options {
                    command += format!(" {key}={value}").as_str();
                }

                let mut additional_options =
                    self.options.additional_options.iter().collect::<Vec<_>>();
                additional_options.sort_by(|left, right| left.key().cmp(right.key()));
                for option in additional_options {
                    command += format!(" {}={}", option.key(), option.value()).as_str();
                }

                command.push('\n');

                Ok(command.into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );
                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Create new subsession.
    pub fn create_subsession(
        &mut self,
        nickname: &str,
        parameters: SessionParameters,
    ) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active { destination, .. } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    %nickname,
                    style = %parameters.style,
                    "create new subsession",
                );
                self.state = SessionState::SubsessionCreatePending { destination };

                let mut command = format!("SESSION ADD STYLE={} ID={nickname} ", parameters.style);

                for (key, value) in parameters.options {
                    command += format!("{key}={value} ").as_str();
                }
                command += "\n";

                Ok(command.into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create subsession, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Handshake stream, either inbound or outbound.
    pub fn handshake_stream(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Uninitialized,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "send handshake for stream",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Handshaking,
                };

                Ok(String::from("HELLO VERSION\n").into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Open virtual stream to `destination`.
    pub fn create_stream(
        &mut self,
        remote_destination: &str,
        options: StreamOptions,
    ) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::info!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    remote_destination = %format!("{}...", &destination[..10]),
                    "open stream to remote destination",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Connect),
                };

                Ok(format!(
                    "STREAM CONNECT ID={} DESTINATION={} FROM_PORT={} TO_PORT={} SILENT=false\n",
                    self.options.nickname, remote_destination, options.src_port, options.dst_port,
                )
                .into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Start accepting a new virtual stream.
    pub fn accept_stream(&mut self) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    "start listening for virtual stream",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Accept),
                };

                Ok(
                    format!("STREAM ACCEPT ID={} SILENT=false\n", self.options.nickname)
                        .into_bytes(),
                )
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Forward inbound virtual streams to a TCP listener listening to `port`.
    pub fn forward_stream(&mut self, port: u16) -> Result<Vec<u8>, ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaked,
            } => {
                tracing::trace!(
                    target: LOG_TARGET,
                    nickname = %self.options.nickname,
                    ?port,
                    "forward incoming connections",
                );
                self.state = SessionState::Active {
                    destination,
                    stream_state: StreamState::Pending(StreamKind::Forward),
                };

                Ok(format!(
                    "STREAM FORWARD ID={} PORT={port} SILENT={}\n",
                    self.options.nickname, self.options.silent_forward,
                )
                .into_bytes())
            }
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot create session, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Handle response from router.
    pub fn handle_response(&mut self, response: &str) -> Result<(), ProtocolError> {
        match std::mem::replace(&mut self.state, SessionState::Poisoned) {
            SessionState::Handshaking => match Response::parse(response) {
                Some(Response::Hello {
                    version: Ok(version),
                }) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        %version,
                        "session handshake done",
                    );
                    self.state = SessionState::Handshaked;

                    Ok(())
                }
                Some(Response::Hello {
                    version: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router session `HELLO`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        "unexpected response from router session `HELLO`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            SessionState::SessionCreatePending => match Response::parse(response) {
                Some(Response::Session {
                    destination: Ok(destination),
                }) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        "session created",
                    );

                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    Ok(())
                }
                Some(Response::Session {
                    destination: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router for `SESSION CREATE`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        "unexpected response from router to `SESSION CREATE`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            SessionState::SubsessionCreatePending { destination } =>
                match Response::parse(response) {
                    Some(Response::Subsession {
                        session_id: Ok(session_id),
                    }) => {
                        tracing::info!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            %session_id,
                            "subsession created",
                        );

                        self.state = SessionState::Active {
                            destination,
                            stream_state: StreamState::Uninitialized,
                        };

                        Ok(())
                    }
                    Some(Response::Subsession {
                        session_id: Err(error),
                    }) => return Err(ProtocolError::Router(error)),
                    None => {
                        tracing::warn!(
                            target: LOG_TARGET,
                            nickname = %self.options.nickname,
                            ?response,
                            "invalid response from router for `SESSION ADD`",
                        );
                        return Err(ProtocolError::InvalidMessage);
                    }
                    Some(response) => {
                        tracing::warn!(
                            nickname = %self.options.nickname,
                            ?response,
                            "unexpected response from router to `SESSION ADD`",
                        );
                        return Err(ProtocolError::InvalidState);
                    }
                },
            SessionState::Active {
                destination,
                stream_state: StreamState::Handshaking,
            } => match Response::parse(response) {
                Some(Response::Hello {
                    version: Ok(version),
                }) => {
                    tracing::trace!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        %version,
                        "stream handshake done",
                    );

                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Handshaked,
                    };

                    Ok(())
                }
                Some(Response::Hello {
                    version: Err(error),
                }) => return Err(ProtocolError::Router(error)),
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        "invalid response from router stream `HELLO`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        "unexpected response from router stream `HELLO`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            SessionState::Active {
                destination,
                stream_state: StreamState::Pending(direction),
            } => match Response::parse(response) {
                Some(Response::Stream { result: Ok(()) }) => {
                    tracing::info!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?direction,
                        "stream status ok",
                    );

                    // after the stream is opened/accepted, the stream is handed off
                    // to user and the stream state can be reset
                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    Ok(())
                }
                Some(Response::Stream { result: Err(error) }) => {
                    // stream failed to open, reset state back to uninitialized
                    self.state = SessionState::Active {
                        destination,
                        stream_state: StreamState::Uninitialized,
                    };

                    return Err(ProtocolError::Router(error));
                }
                None => {
                    tracing::warn!(
                        target: LOG_TARGET,
                        nickname = %self.options.nickname,
                        ?response,
                        ?direction,
                        "invalid response from router to `STREAM CREATE`",
                    );
                    return Err(ProtocolError::InvalidMessage);
                }
                Some(response) => {
                    tracing::warn!(
                        nickname = %self.options.nickname,
                        ?response,
                        ?direction,
                        "unexpected response from router to `STREAM CREATE`",
                    );
                    return Err(ProtocolError::InvalidState);
                }
            },
            state => {
                tracing::warn!(
                    target: LOG_TARGET,
                    ?state,
                    "cannot handle response, invalid state",
                );

                debug_assert!(false);
                Err(ProtocolError::InvalidState)
            }
        }
    }

    /// Get reference to [`SessionController`]'s destination.
    ///
    /// Panics if called before the session is active.
    pub fn destination(&self) -> &str {
        let SessionState::Active { destination, .. } = &self.state else {
            panic!("invalid state");
        };

        &destination
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LeaseSetClientAuth, SessionOption, MAX_ADDITIONAL_SESSION_OPTIONS,
        MAX_LEASE_SET_CLIENT_AUTHS, MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH,
        MAX_LEASE_SET_CLIENT_NAME_LENGTH, MAX_SESSION_OPTION_KEY_LENGTH,
        MAX_SESSION_OPTION_VALUE_LENGTH,
    };

    fn handshaked_controller(options: SessionOptions) -> SessionController {
        let mut controller = SessionController::new(options).unwrap();
        assert_eq!(
            controller.handshake_session(),
            Ok(b"HELLO VERSION\n".to_vec())
        );
        controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").unwrap();
        controller
    }

    fn create_stream_command(options: SessionOptions) -> Result<String, ProtocolError> {
        let mut controller = handshaked_controller(options);
        let command = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        })?;
        Ok(String::from_utf8(command).unwrap())
    }

    fn option_count(command: &str, key: &str) -> usize {
        command
            .split_whitespace()
            .filter(|token| token.strip_prefix(key).is_some_and(|value| value.starts_with('=')))
            .count()
    }

    #[test]
    fn session_create_serializes_typed_options() {
        let command = create_stream_command(SessionOptions::default()).unwrap();
        assert!(command.ends_with(
            "inbound.lengthVariance=0 inbound.backupQuantity=0 outbound.lengthVariance=0 outbound.backupQuantity=0 SIGNATURE_TYPE=7\n"
        ));
        assert_eq!(option_count(&command, "SIGNATURE_TYPE"), 1);

        let command = create_stream_command(SessionOptions {
            signature_type: 11,
            inbound_len_variance: -2,
            outbound_len_variance: 3,
            inbound_backup_quantity: 4,
            outbound_backup_quantity: 5,
            ..Default::default()
        })
        .unwrap();

        assert_eq!(option_count(&command, "SIGNATURE_TYPE"), 1);
        assert!(command.contains("SIGNATURE_TYPE=11"));
        assert!(command.contains("inbound.lengthVariance=-2"));
        assert!(command.contains("outbound.lengthVariance=3"));
        assert!(command.contains("inbound.backupQuantity=4"));
        assert!(command.contains("outbound.backupQuantity=5"));
        assert_eq!(option_count(&command, "inbound.lengthVariance"), 1);
        assert_eq!(option_count(&command, "outbound.lengthVariance"), 1);
        assert_eq!(option_count(&command, "inbound.backupQuantity"), 1);
        assert_eq!(option_count(&command, "outbound.backupQuantity"), 1);
    }

    #[test]
    fn session_create_serializes_sorted_additional_options() {
        let mut options = SessionOptions::default();
        options.add_session_option("i2cp.zeta", "last").unwrap();
        options.add_session_option("i2cp.alpha", "first").unwrap();

        let command = create_stream_command(options).unwrap();
        assert!(command.ends_with("SIGNATURE_TYPE=7 i2cp.alpha=first i2cp.zeta=last\n"));
        assert_eq!(command.matches('\n').count(), 1);
    }

    #[test]
    fn additional_options_reject_duplicates_and_reserved_keys() {
        let mut options = SessionOptions::default();
        options.add_session_option("i2cp.custom", "one").unwrap();
        assert!(options.add_session_option("i2cp.custom", "two").is_err());

        for key in [
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
            "inbound.length",
            "inbound.quantity",
            "outbound.length",
            "outbound.quantity",
            "inbound.lengthVariance",
            "inbound.backupQuantity",
            "outbound.lengthVariance",
            "outbound.backupQuantity",
            "i2cp.dontPublishLeaseSet",
            "i2cp.leaseSetEncType",
        ] {
            assert!(
                options.add_session_option(key, "value").is_err(),
                "reserved key was accepted: {key}"
            );
        }
    }

    #[test]
    fn additional_options_reject_injection_and_enforce_bounds() {
        for (key, value) in [
            ("", "value"),
            ("i2cp.bad=key", "value"),
            ("i2cp.bad key", "value"),
            ("i2cp.bad\nkey", "value"),
            ("i2cp.bad", "value with spaces"),
            ("i2cp.bad", "value\nwith-newline"),
            ("i2cp.bad", "value\0with-nul"),
            ("i2cp.bad", "value=another-token"),
            ("i2cp.bad", "value\\another-token"),
        ] {
            assert!(SessionOption::new(key, value).is_err());
        }

        let key = "k".repeat(MAX_SESSION_OPTION_KEY_LENGTH);
        let value = "v".repeat(MAX_SESSION_OPTION_VALUE_LENGTH);
        let mut options = SessionOptions::default();
        options.add_session_option(key, value).unwrap();
        assert!(SessionOption::new("k".repeat(MAX_SESSION_OPTION_KEY_LENGTH + 1), "v").is_err());
        assert!(SessionOption::new("k", "v".repeat(MAX_SESSION_OPTION_VALUE_LENGTH + 1)).is_err());

        for index in 1..MAX_ADDITIONAL_SESSION_OPTIONS {
            options.add_session_option(format!("i2cp.option{index}"), "value").unwrap();
        }
        assert!(options.add_session_option("i2cp.tooMany", "value").is_err());
        assert_eq!(
            options.additional_options.len(),
            MAX_ADDITIONAL_SESSION_OPTIONS
        );
    }

    #[test]
    fn invalid_public_option_collection_does_not_change_controller_state() {
        let mut controller = handshaked_controller(SessionOptions::default());
        let option = SessionOption::new("i2cp.option", "value").unwrap();
        controller.options.additional_options = vec![option; MAX_ADDITIONAL_SESSION_OPTIONS + 1];

        let result = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        });
        assert_eq!(result, Err(ProtocolError::InvalidOption));
        assert_eq!(controller.state, SessionState::Handshaked);
    }

    #[test]
    fn session_options_debug_redacts_private_and_additional_values() {
        let private_key = "persistent-private-key-fixture";
        let additional_value = "additional-secret-fixture";
        let mut options = SessionOptions {
            destination: DestinationKind::Persistent {
                private_key: private_key.to_string(),
            },
            lease_set_key: Some("lease-set-key-fixture".to_string()),
            lease_set_private_key: Some("lease-set-private-key-fixture".to_string()),
            lease_set_secret: Some("lease-set-secret-fixture".to_string()),
            lease_set_signing_private_key: Some("lease-set-signing-key-fixture".to_string()),
            ..Default::default()
        };
        options.add_session_option("i2cp.custom", additional_value).unwrap();

        let debug = format!("{options:?}");
        for secret in [
            private_key,
            additional_value,
            "lease-set-key-fixture",
            "lease-set-private-key-fixture",
            "lease-set-secret-fixture",
            "lease-set-signing-key-fixture",
        ] {
            assert!(!debug.contains(secret));
        }
        assert!(debug.contains("<redacted>"));
    }

    #[test]
    fn open_virtual_stream() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

        // handshake virtual stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn accept_virtual_stream() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

        // handshake virtual stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.accept_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Accept),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn dont_publish_lease_set() {
        let mut controller = SessionController::new(SessionOptions {
            publish: false,
            ..Default::default()
        })
        .unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(command.contains("i2cp.dontPublishLeaseSet=true"));
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

        // handshake virtual stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn stream_fails_to_open() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

        // handshake virtual stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect failure
        assert!(controller.handle_response("STREAM STATUS RESULT=CANT_REACH_PEER\n").is_err());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // try to open another stream
        assert!(controller.handshake_stream().is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaking,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle handshake response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());

        let SessionState::Active {
            stream_state: StreamState::Handshaked,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // create virtual stream
        assert!(controller.create_stream("destination", Default::default()).is_ok(),);

        let SessionState::Active {
            stream_state: StreamState::Pending(StreamKind::Connect),
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };

        // handle connect response
        assert!(controller.handle_response("STREAM STATUS RESULT=OK\n").is_ok());

        // stream state is reset after it has been opened/accepted
        let SessionState::Active {
            stream_state: StreamState::Uninitialized,
            ..
        } = controller.state
        else {
            panic!("invalid state");
        };
    }

    #[test]
    fn create_primary_and_subsession() {
        let mut controller = SessionController::new(Default::default()).unwrap();

        // handshake session
        assert_eq!(controller.state, SessionState::Uninitialized);
        assert_eq!(
            controller.handshake_session(),
            Ok(String::from("HELLO VERSION\n").into_bytes())
        );
        assert_eq!(controller.state, SessionState::Handshaking);

        // handle response
        assert!(controller.handle_response("HELLO REPLY RESULT=OK VERSION=3.3\n").is_ok());
        assert_eq!(controller.state, SessionState::Handshaked);

        // create session
        let parameters = SessionParameters {
            style: "PRIMARY".to_string(),
            options: Vec::new(),
        };
        let command = controller.create_session(parameters).unwrap();
        let command = std::str::from_utf8(&command).unwrap();
        assert!(!command.contains("i2cp.dontPublishLeaseSet=true"));
        assert_eq!(controller.state, SessionState::SessionCreatePending);

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK DESTINATION=I2P_DESTINATION\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }

        assert!(controller
            .create_subsession(
                "test",
                SessionParameters {
                    style: "STREAM".to_string(),
                    options: Vec::new()
                }
            )
            .is_ok());

        let SessionState::SubsessionCreatePending { .. } = controller.state else {
            panic!("invalid state");
        };

        // handle response and create virtual stream
        assert!(controller
            .handle_response("SESSION STATUS RESULT=OK ID=\"lS24mtNyeNVMf2bZ\" MESSAGE=\"ADD lS24mtNyeNVMf2bZ\"\n\n")
            .is_ok());

        match &controller.state {
            SessionState::Active { destination, .. }
                if destination.as_str() == "I2P_DESTINATION" => {}
            state => panic!("invalid state: {state:?}"),
        }
    }

    #[test]
    fn session_create_serializes_leaseset_typed_options() {
        // Independent reference fixture: these literals come from the Java/I2CP vocabulary and
        // are intentionally not derived from the serializer's emitted strings.
        const CANONICAL_KEYS: &[&str] = &[
            "i2cp.encryptLeaseSet",
            "i2cp.leaseSetAuthType",
            "i2cp.leaseSetBlindedType",
            "i2cp.leaseSetType",
            "i2cp.leaseSetKey",
            "i2cp.leaseSetPrivateKey",
            "i2cp.leaseSetSecret",
            "i2cp.leaseSetSigningPrivateKey",
            "i2cp.leaseSetClient.dh.0",
            "i2cp.leaseSetClient.psk.0",
        ];

        // Default wire must remain unchanged when encrypted/auth settings are unused.
        let default_command = create_stream_command(SessionOptions::default()).unwrap();
        for key in CANONICAL_KEYS {
            assert!(!default_command.contains(key), "default emitted {key}");
        }

        // Non-default encrypt + auth + blinded + type + secrets.
        let options = SessionOptions {
            encrypt_lease_set: true,
            lease_set_auth_type: 1,
            lease_set_blinded_type: 10,
            lease_set_type: 3,
            lease_set_key: Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string()),
            lease_set_private_key: Some("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=".to_string()),
            lease_set_secret: Some("c2VjcmV0LXZhbHVlLWZpeHR1cmU=".to_string()),
            lease_set_signing_private_key: Some(
                "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=".to_string(),
            ),
            ..Default::default()
        };

        let command = create_stream_command(options).unwrap();
        assert_eq!(option_count(&command, "i2cp.encryptLeaseSet"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetAuthType"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetBlindedType"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetType"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetKey"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetPrivateKey"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetSecret"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetSigningPrivateKey"), 1);
        assert!(command.contains("i2cp.encryptLeaseSet=true"));
        assert!(command.contains("i2cp.leaseSetAuthType=1"));
        assert!(command.contains("i2cp.leaseSetBlindedType=10"));
        assert!(command.contains("i2cp.leaseSetType=3"));
        assert!(command.contains("i2cp.leaseSetKey=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
        assert!(command
            .contains("i2cp.leaseSetPrivateKey=BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="));
        assert!(command.contains("i2cp.leaseSetSecret=c2VjcmV0LXZhbHVlLWZpeHR1cmU="));
        assert!(command.contains(
            "i2cp.leaseSetSigningPrivateKey=CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC="
        ));
        assert!(!command.contains("i2cp.leaseSetPrivKey="));
        assert!(!command.contains("i2cp.leaseSetSigningPrivKey="));
        assert_eq!(command.matches('\n').count(), 1);
    }

    #[test]
    fn session_create_serializes_leaseset_client_auths_deterministically() {
        let mut options = SessionOptions::default();
        const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

        // Insert both modes out of order to verify deterministic per-mode numbering.
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("zulu", KEY).unwrap())
            .unwrap();
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("zulu", KEY).unwrap())
            .unwrap();
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("alice", KEY).unwrap())
            .unwrap();
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", KEY).unwrap())
            .unwrap();

        let command = create_stream_command(options).unwrap();
        assert_eq!(option_count(&command, "i2cp.leaseSetClient.dh.0"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetClient.dh.1"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetClient.psk.0"), 1);
        assert_eq!(option_count(&command, "i2cp.leaseSetClient.psk.1"), 1);
        assert!(command.contains(&format!("i2cp.leaseSetClient.dh.0=YWxpY2U=:{KEY}")));
        assert!(command.contains(&format!("i2cp.leaseSetClient.dh.1=enVsdQ==:{KEY}")));
        assert!(command.contains(&format!("i2cp.leaseSetClient.psk.0=YWxpY2U=:{KEY}")));
        assert!(command.contains(&format!("i2cp.leaseSetClient.psk.1=enVsdQ==:{KEY}")));
        // Second creation must be identical (deterministic) modulo random nickname.
        let mut options2 = SessionOptions::default();
        options2
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("alice", KEY).unwrap())
            .unwrap();
        options2
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("alice", KEY).unwrap())
            .unwrap();
        options2
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("zulu", KEY).unwrap())
            .unwrap();
        options2
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("zulu", KEY).unwrap())
            .unwrap();
        let command2 = create_stream_command(options2).unwrap();
        // Compare suffix after SIGNATURE_TYPE to ignore random ID.
        let suffix = |cmd: &str| cmd.split("SIGNATURE_TYPE=7").nth(1).unwrap_or("").to_string();
        assert_eq!(suffix(&command), suffix(&command2));
    }

    #[test]
    fn lease_set_client_auth_rejects_duplicates_and_bounds() {
        const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let mut options = SessionOptions::default();
        let alice = LeaseSetClientAuth::dh("alice", KEY).unwrap();
        options.add_lease_set_client_auth(alice.clone()).unwrap();
        assert!(options.add_lease_set_client_auth(alice).is_err());
        // The client name is the logical identity within one authorization mode, regardless of
        // whether the key material differs.
        assert!(options
            .add_lease_set_client_auth(
                LeaseSetClientAuth::dh("alice", "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE=")
                    .unwrap()
            )
            .is_err());
        // The same display name may be used once in each distinct reference namespace.
        options
            .add_lease_set_client_auth(LeaseSetClientAuth::psk("alice", KEY).unwrap())
            .unwrap();

        // Max count
        let mut many = SessionOptions::default();
        for i in 0..MAX_LEASE_SET_CLIENT_AUTHS {
            many.add_lease_set_client_auth(
                LeaseSetClientAuth::dh(format!("client-{i}"), KEY).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(
            many.lease_set_client_auths.len(),
            MAX_LEASE_SET_CLIENT_AUTHS
        );
        assert!(many
            .add_lease_set_client_auth(LeaseSetClientAuth::dh("overflow", KEY).unwrap())
            .is_err());

        // Direct vec mutation with duplicate must be caught by validation before wire.
        let mut controller = handshaked_controller(SessionOptions::default());
        let dup = LeaseSetClientAuth::dh("duplicate", KEY).unwrap();
        controller.options.lease_set_client_auths = vec![dup.clone(), dup];
        let result = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        });
        assert_eq!(result, Err(ProtocolError::InvalidOption));
        assert_eq!(controller.state, SessionState::Handshaked);
    }

    #[test]
    fn lease_set_client_auth_rejects_malformed_and_injection() {
        for key in [
            "",
            "not-base64!",
            "has space",
            "has\nnewline",
            "AAA=AAA", // = in middle
            "value=with-equals",
            "value with spaces",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+=", // standard alphabet is not I2P base64
        ] {
            assert!(
                LeaseSetClientAuth::dh("client", key).is_err(),
                "malformed key accepted: {key}"
            );
        }
        let long = "A".repeat(MAX_LEASE_SET_CLIENT_AUTH_KEY_LENGTH + 1);
        assert!(LeaseSetClientAuth::dh("client", long).is_err());
        assert!(
            LeaseSetClientAuth::dh("", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").is_err()
        );
        let long_name = "n".repeat(MAX_LEASE_SET_CLIENT_NAME_LENGTH + 1);
        assert!(
            LeaseSetClientAuth::dh(long_name, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                .is_err()
        );
        assert!(
            LeaseSetClientAuth::dh("client\n", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                .is_err()
        );
        assert!(
            LeaseSetClientAuth::dh("client", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB=")
                .is_err()
        );
        // Oversized secret
        let options = SessionOptions {
            lease_set_secret: Some("A".repeat(crate::MAX_LEASE_SET_SECRET_LENGTH + 1)),
            ..Default::default()
        };
        assert!(SessionController::new(options).is_err());
    }

    #[test]
    fn lease_set_typed_generic_conflict_rejects() {
        let mut options = SessionOptions {
            encrypt_lease_set: true,
            ..Default::default()
        };
        assert!(options.add_session_option("i2cp.encryptLeaseSet", "true").is_err());

        let mut options = SessionOptions {
            lease_set_auth_type: 1,
            ..Default::default()
        };
        assert!(options.add_session_option("i2cp.leaseSetAuthType", "1").is_err());

        let mut options = SessionOptions::default();
        options
            .add_lease_set_client_auth(
                LeaseSetClientAuth::dh("client", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                    .unwrap(),
            )
            .unwrap();
        assert!(options.add_session_option("i2cp.leaseSetClient.dh.0", "value").is_err());
        assert!(options.add_session_option("i2cp.leaseSetClient.psk.99", "value").is_err());

        // Also validate that generic options cannot use reserved LeaseSet keys at all.
        assert!(crate::SessionOption::new("i2cp.leaseSetSecret", "value").is_err());
        assert!(crate::SessionOption::new("i2cp.encryptLeaseSet", "true").is_err());
        assert!(crate::SessionOption::new("i2cp.leaseSetAuthType", "1").is_err());
        for key in [
            "i2cp.leaseSetPrivateKey",
            "i2cp.leaseSetSigningPrivateKey",
            "i2cp.leaseSetPrivKey",
            "i2cp.leaseSetSigningPrivKey",
            "i2cp.leaseSetClient.dh.0",
            "i2cp.leaseSetClient.psk.0",
        ] {
            assert!(
                crate::SessionOption::new(key, "value").is_err(),
                "accepted {key}"
            );
        }
    }

    #[test]
    fn lease_set_numeric_domains_match_reference_wire_types() {
        for auth_type in [0, 1, 2] {
            let options = SessionOptions {
                lease_set_auth_type: auth_type,
                ..Default::default()
            };
            let command = create_stream_command(options).unwrap();
            if auth_type == 0 {
                assert_eq!(option_count(&command, "i2cp.leaseSetAuthType"), 0);
            } else {
                assert_eq!(option_count(&command, "i2cp.leaseSetAuthType"), 1);
                assert!(command.contains(&format!("i2cp.leaseSetAuthType={auth_type}")));
            }
        }
        for blinded_type in [0, 1, 12, u16::MAX as usize] {
            let options = SessionOptions {
                lease_set_blinded_type: blinded_type,
                ..Default::default()
            };
            let command = create_stream_command(options).unwrap();
            if blinded_type == 0 {
                assert_eq!(option_count(&command, "i2cp.leaseSetBlindedType"), 0);
            } else {
                assert!(command.contains(&format!("i2cp.leaseSetBlindedType={blinded_type}")));
            }
        }
        for lease_set_type in [1, 3, 5, 7, u8::MAX as usize] {
            let options = SessionOptions {
                lease_set_type,
                ..Default::default()
            };
            let command = create_stream_command(options).unwrap();
            if lease_set_type != 1 {
                assert!(command.contains(&format!("i2cp.leaseSetType={lease_set_type}")));
            }
        }

        for (field, value) in [
            ("auth", 3usize),
            ("blinded", u16::MAX as usize + 1),
            ("lease_set", 0usize),
            ("lease_set", u8::MAX as usize + 1),
        ] {
            let mut options = SessionOptions::default();
            match field {
                "auth" => options.lease_set_auth_type = value,
                "blinded" => options.lease_set_blinded_type = value,
                "lease_set" => options.lease_set_type = value,
                _ => unreachable!(),
            }
            assert!(
                SessionController::new(options).is_err(),
                "accepted {field}={value}"
            );
        }

        // Numeric LeaseSet keys are owned by the typed API, so malformed textual spellings cannot
        // reach the wire through the generic option path either.
        for value in ["-1", "256", " 3", "3 ", "+3", "0x5", "decimal"] {
            assert!(SessionOption::new("i2cp.leaseSetType", value).is_err());
        }

        let mut controller = handshaked_controller(SessionOptions::default());
        controller.options.lease_set_blinded_type = u16::MAX as usize + 1;
        let result = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        });
        assert_eq!(result, Err(ProtocolError::InvalidOption));
        assert_eq!(controller.state, SessionState::Handshaked);
    }

    #[test]
    fn lease_set_invalid_fails_before_bytes_no_downgrade() {
        // Invalid auth type
        let options = SessionOptions {
            lease_set_auth_type: 99,
            ..Default::default()
        };
        assert!(SessionController::new(options).is_err());

        // Malformed base64 secret
        let options = SessionOptions {
            lease_set_secret: Some("not-base64!".to_string()),
            ..Default::default()
        };
        let mut controller = handshaked_controller(SessionOptions::default());
        controller.options = options;
        let result = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        });
        assert_eq!(result, Err(ProtocolError::InvalidOption));
        assert_eq!(controller.state, SessionState::Handshaked);
        // Must not have emitted a weaker command (no bytes returned).

        // Invalid key length
        let options = SessionOptions {
            lease_set_key: Some("".to_string()),
            ..Default::default()
        };
        assert!(SessionController::new(options).is_err());

        // Oversized client auth via direct vec
        let mut controller = handshaked_controller(SessionOptions::default());
        let many =
            vec![
                LeaseSetClientAuth::dh("client", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                    .unwrap();
                MAX_LEASE_SET_CLIENT_AUTHS + 1
            ];
        controller.options.lease_set_client_auths = many;
        let result = controller.create_session(SessionParameters {
            style: "STREAM".to_string(),
            options: Vec::new(),
        });
        assert_eq!(result, Err(ProtocolError::InvalidOption));
        assert_eq!(controller.state, SessionState::Handshaked);
    }

    #[test]
    fn lease_set_secret_redaction() {
        let mut options = SessionOptions {
            lease_set_key: Some("lease-key-secret".to_string()),
            lease_set_private_key: Some("lease-priv-secret".to_string()),
            lease_set_secret: Some("lease-secret-value".to_string()),
            lease_set_signing_private_key: Some("lease-signing-secret".to_string()),
            ..Default::default()
        };
        // Use valid base64-like secrets to pass validation but still check redaction
        // Here we bypass validation for redaction check by using directly set values that are not validated until controller new.
        // Instead test Debug directly without going through controller validation.
        options.lease_set_key = Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string());
        options.lease_set_private_key =
            Some("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=".to_string());
        options.lease_set_secret = Some("c2VjcmV0LXZhbHVlLWZpeHR1cmU=".to_string());
        options.lease_set_signing_private_key =
            Some("CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=".to_string());
        options
            .add_lease_set_client_auth(
                LeaseSetClientAuth::dh("client", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                    .unwrap(),
            )
            .unwrap();

        let debug = format!("{options:?}");
        for secret in [
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=",
            "c2VjcmV0LXZhbHVlLWZpeHR1cmU=",
            "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=",
            "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD=",
        ] {
            assert!(!debug.contains(secret), "secret leaked in Debug: {secret}");
        }
        assert!(debug.contains("<redacted>"));

        // Also check LeaseSetClientAuth Debug
        let auth =
            LeaseSetClientAuth::psk("client", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
                .unwrap();
        let auth_debug = format!("{auth:?}");
        assert!(!auth_debug.contains("AAAAAAAA"));
        assert!(auth_debug.contains("<redacted>"));

        // Error path must not leak: InvalidOption Display is generic.
        let err = LeaseSetClientAuth::dh("client", "bad!").unwrap_err();
        let err_str = format!("{err}");
        assert!(!err_str.contains("bad!"));
    }
}
