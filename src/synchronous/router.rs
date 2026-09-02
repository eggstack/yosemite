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

#![cfg(feature = "sync")]

use crate::{
    error::Error, options::SAMV3_TCP_PORT, proto::router::RouterApiController,
    synchronous::read_response,
};

use std::{io::Write, net::TcpStream};

/// ## Router API.
///
/// `RouterApi` provides SAM functionality unrelated to active sessions. `RouterApi` connects to the
/// router via the default SAMV3 TCP port (7656) but this can be overridden by calling
/// [`RouterApi::new()`] with a custom port.
pub struct RouterApi {
    /// SAMv3 TCP port.
    port: u16,
}

impl Default for RouterApi {
    fn default() -> Self {
        Self {
            port: SAMV3_TCP_PORT,
        }
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn public_sync_api_passes_selected_signature_type_to_router() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut command = String::new();

            reader.read_line(&mut command).unwrap();
            assert_eq!(command, "HELLO VERSION\n");
            stream.write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n").unwrap();

            command.clear();
            reader.read_line(&mut command).unwrap();
            assert_eq!(command, "DEST GENERATE SIGNATURE_TYPE=11\n");
            stream.write_all(b"DEST REPLY PUB=destination PRIV=private\n").unwrap();
        });

        let generated = RouterApi::new(port).generate_destination_with_signature_type(11).unwrap();

        assert_eq!(
            generated,
            ("destination".to_string(), "private".to_string())
        );
        server.join().unwrap();
    }

    #[test]
    fn public_sync_default_api_requests_ed25519() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut command = String::new();

            reader.read_line(&mut command).unwrap();
            assert_eq!(command, "HELLO VERSION\n");
            stream.write_all(b"HELLO REPLY RESULT=OK VERSION=3.3\n").unwrap();

            command.clear();
            reader.read_line(&mut command).unwrap();
            assert_eq!(command, "DEST GENERATE SIGNATURE_TYPE=7\n");
            stream.write_all(b"DEST REPLY PUB=destination PRIV=private\n").unwrap();
        });

        RouterApi::new(port).generate_destination().unwrap();
        server.join().unwrap();
    }
}

impl RouterApi {
    /// Create new [`RouterApi`] and connect router over `port`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yosemite::RouterApi;
    ///
    /// fn main() -> yosemite::Result<()> {
    ///     let (destination, private_key) = RouterApi::new(8888).generate_destination()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl RouterApi {
    /// Attempt to look up the the destination associated with `name`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yosemite::RouterApi;
    ///
    /// fn main() -> yosemite::Result<()> {
    ///     let destination = RouterApi::default().lookup_name("host.i2p")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn lookup_name(&self, name: &str) -> crate::Result<String> {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))?;

        // send handhake to router
        let command = controller.handshake_router_api()?;
        stream.write_all(&command)?;

        // read handshake response
        let response = read_response(&mut stream).ok_or(Error::Malformed)?;
        controller.handle_response(&response)?;

        // lookup hostname
        let command = controller.lookup_name(name)?;
        stream.write_all(&command)?;

        // handle hostname lookup response
        let response = read_response(&mut stream).ok_or(Error::Malformed)?;
        controller.handle_response(&response)?;

        Ok(controller.destination())
    }

    /// Generate destination.
    ///
    /// The first element in the returned tuple is a base64-encoded destination which can used by
    /// other destinations to connect to the generated destination. The second element in the tuple
    /// is the private key of the destination which can be used to create the destination using
    /// [`DestinationKind::Persistent`](crate::options::DestinationKind).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use yosemite::RouterApi;
    ///
    /// fn main() -> yosemite::Result<()> {
    ///     let (destination, private_key) = RouterApi::default().generate_destination()?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn generate_destination(&self) -> crate::Result<(String, String)> {
        self.generate_destination_with_command(|controller| controller.generate_destination())
    }

    /// Generate a destination using the requested SAM signature type.
    ///
    /// The signature type is passed directly to the router. Router support and validity are
    /// reported through the returned result; this method does not retry with the default type.
    pub fn generate_destination_with_signature_type(
        &self,
        signature_type: u16,
    ) -> crate::Result<(String, String)> {
        self.generate_destination_with_command(|controller| {
            controller.generate_destination_with_signature_type(signature_type)
        })
    }

    fn generate_destination_with_command<F>(
        &self,
        generate_command: F,
    ) -> crate::Result<(String, String)>
    where
        F: FnOnce(&mut RouterApiController) -> Result<Vec<u8>, crate::ProtocolError>,
    {
        let mut controller = RouterApiController::new();
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", self.port))?;

        // send handhake to router
        let command = controller.handshake_router_api()?;
        stream.write_all(&command)?;

        // read handshake response
        let response = read_response(&mut stream).ok_or(Error::Malformed)?;
        controller.handle_response(&response)?;

        // generate destination
        let command = generate_command(&mut controller)?;
        stream.write_all(&command)?;

        // read destination generation response
        let response = read_response(&mut stream).ok_or(Error::Malformed)?;
        controller.handle_response(&response)?;

        Ok(controller.generated_destination())
    }
}
