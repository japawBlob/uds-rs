//!
//! Backend layer for the uds-rs library.
//!
//! Currently built using tokio_socketcan_isotp library, the process should be similar for
//! different network protocols and even runtimes, but it is currently tested only on tokio_socketcan_isotp and you knowledge may vary.
//!
//! To provide your own backend communication just rewrite the read, write and socket creation process to use your own API, and you should be good to go.
//!

use std::time::Duration;
pub use tokio_socketcan_isotp::{
    Error, ExtendedId, FlowControlOptions, Id, IsoTpBehaviour, IsoTpOptions, LinkLayerOptions,
    StandardId, TxFlags,
};

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum UdsCommunicationError {
    FailedToFindCanDevice,
    SocketCanIOError,
    StdIOError,
    GeneralError,
    NotImplementedError,
    SocketCreationError,
}

impl From<Error> for UdsCommunicationError {
    fn from(err: Error) -> Self {
        match err {
            Error::Io { .. } => UdsCommunicationError::SocketCanIOError,
            Error::Lookup { .. } => UdsCommunicationError::FailedToFindCanDevice,
        }
    }
}

impl From<std::io::Error> for UdsCommunicationError {
    fn from(_err: std::io::Error) -> Self {
        UdsCommunicationError::StdIOError
    }
}

pub struct UdsSocket {
    isotp_socket: tokio_socketcan_isotp::IsoTpSocket,
}

impl UdsSocket {

    pub fn new(
        ifname: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
    ) -> Result<UdsSocket, UdsCommunicationError> {
        Ok(UdsSocket {
            isotp_socket: tokio_socketcan_isotp::IsoTpSocket::open(ifname, src, dst)?,
        })
    }

    pub fn new_vw(
        ifname: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
    ) -> Result<UdsSocket, UdsCommunicationError> {
        let mut behav = IsoTpBehaviour::CAN_ISOTP_RX_PADDING;
        behav.set(IsoTpBehaviour::CAN_ISOTP_TX_PADDING, true);

        let tp_options =
            IsoTpOptions::new(behav, Duration::from_secs(1), u8::MAX, 0x55, 0xAA, u8::MAX);

        let mut behav = IsoTpBehaviour::CAN_ISOTP_RX_PADDING;
        behav.set(IsoTpBehaviour::CAN_ISOTP_TX_PADDING, true);
        let mut options = None;

        match tp_options {
            Ok(mut o) => {
                o.set_flags(behav);
                options = Some(o);
            }
            Err(_) => println!("Cannot set options!"),
        }
        let uds_socket = UdsSocket::new_with_opts(&ifname, src, dst, options, None, None)?;
        return Ok(uds_socket);
    }

    pub fn new_with_opts(
        ifname: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
        isotp_options: Option<IsoTpOptions>,
        rx_flow_control_options: Option<FlowControlOptions>,
        link_layer_options: Option<LinkLayerOptions>,
    ) -> Result<UdsSocket, UdsCommunicationError> {
        Ok(UdsSocket {
            isotp_socket: tokio_socketcan_isotp::IsoTpSocket::open_with_opts(
                ifname,
                src,
                dst,
                isotp_options,
                rx_flow_control_options,
                link_layer_options,
            )?,
        })
    }

    pub async fn send(&self, payload: &[u8]) -> Result<(), UdsCommunicationError> {
        Ok(self.isotp_socket.write_packet(payload)?.await?)
    }
    pub async fn receive(&self) -> Result<Vec<u8>, UdsCommunicationError> {
        Ok(self.isotp_socket.read_packet()?.await?)
    }
}
