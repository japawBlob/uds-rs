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

#[derive(Default)]
pub struct UdsSocketOptions {
    pub isotp_options: Option<IsoTpOptions>,
    pub rx_flow_control_options: Option<FlowControlOptions>,
    pub link_layer_options: Option<LinkLayerOptions>,
}

impl UdsSocketOptions {
    pub fn is_default(&self) -> bool {
        self.isotp_options.is_none()
            && self.rx_flow_control_options.is_none()
            && self.link_layer_options.is_none()
    }

    pub fn vw() -> Result<Self, UdsCommunicationError> {
        let mut initial_flags = IsoTpBehaviour::CAN_ISOTP_RX_PADDING;
        initial_flags.set(IsoTpBehaviour::CAN_ISOTP_TX_PADDING, true);

        let mut isotp_options = IsoTpOptions::new(
            initial_flags,
            Duration::from_secs(1),
            u8::MAX,
            0x55,
            0xAA,
            u8::MAX,
        )
        .map_err(|_| UdsCommunicationError::SocketCreationError)?;

        let mut runtime_flags = IsoTpBehaviour::CAN_ISOTP_RX_PADDING;
        runtime_flags.set(IsoTpBehaviour::CAN_ISOTP_TX_PADDING, true);
        isotp_options.set_flags(runtime_flags);

        Ok(Self {
            isotp_options: Some(isotp_options),
            rx_flow_control_options: None,
            link_layer_options: None,
        })
    }
}

impl UdsSocket {
    pub fn new(
        ifname: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
        options: UdsSocketOptions,
    ) -> Result<UdsSocket, UdsCommunicationError> {
        let src = src.into();
        let dst = dst.into();

        if options.is_default() {
            Ok(UdsSocket {
                isotp_socket: tokio_socketcan_isotp::IsoTpSocket::open(ifname, src, dst)?,
            })
        } else {
            Ok(UdsSocket {
                isotp_socket: tokio_socketcan_isotp::IsoTpSocket::open_with_opts(
                    ifname,
                    src,
                    dst,
                    options.isotp_options,
                    options.rx_flow_control_options,
                    options.link_layer_options,
                )?,
            })
        }
    }

    pub async fn send(&self, payload: &[u8]) -> Result<(), UdsCommunicationError> {
        Ok(self.isotp_socket.write_packet(payload).await?)
    }
    pub async fn receive(&self) -> Result<Vec<u8>, UdsCommunicationError> {
        Ok(self.isotp_socket.read_packet().await?)
    }
}
