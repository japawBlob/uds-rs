#![deny(clippy::all)]
#![allow(dead_code)]
//!
//! # Uds.rs
mod communication;

mod clear_diagnostic_information;
mod diagnostic_session_control;
mod ecu_reset;
mod read_data_by_identifier;
mod read_dtc_information;
mod read_memory_by_address;
mod uds_definitions;
mod write_data_by_identifier;

use std::time::Duration;

pub use crate::uds::communication::*;
pub use crate::uds::ecu_reset::*;
pub use crate::uds::read_data_by_identifier::*;
pub use crate::uds::read_dtc_information::*;
pub use crate::uds::read_memory_by_address::*;
pub use crate::uds::uds_definitions::*;
pub use crate::uds::write_data_by_identifier::*;
use diagnostic_session_control::DiagnosticSessionControlResponse;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use thiserror::Error;

pub type EcuResponseResult = Result<UdsResponse, UdsError>;

/// All possible services containing responses
/// DataFormat represents wether the parsing into response struct was succesful
#[derive(Debug, PartialEq)]
pub enum UdsResponse {
    EcuReset(DataFormat<EcuResetResponse>),
    ReadDataByIdentifier(DataFormat<ReadDataByIdentifierResponse>),
    ReadMemoryByAddress(DataFormat<ReadMemoryByAddressResponse>),
    ReadDTCInformation(DataFormat<ReadDTCInformationResponse>),
    ClearDiagnosticInformation,
    WriteDataByIdentifier(DataFormat<WriteDataByIdentifierResponse>),
    DiagnosticSessionControl(DataFormat<DiagnosticSessionControlResponse>),
}

/// If program was able to parse received data, the response struct will be stored in Parsed.
/// If parsing was not successful, the Raw will contain all received data, without first byte (SID)
/// which is encoded in UdsResponse Enum
#[derive(Debug, PartialEq)]
pub enum DataFormat<T> {
    Parsed(T),
    Raw(Vec<u8>),
}

/// Containing possible errors and negative responses
#[derive(Error, Debug, PartialEq)]
pub enum UdsError {
    #[error(
        "Response received does not have expected SID. Expected: {expected:x}, Received: {received:x}"
    )]
    SidMismatch {
        expected: u8,
        received: u8,
        raw_message: Vec<u8>,
    },
    #[error(
        "Sent and received data identifier don't match. Expected: {expected:x}, Received: {received:x}"
    )]
    DidMismatch {
        expected: u16,
        received: u16,
        raw_message: Vec<u8>,
    },
    #[error(
        "Received message doesn't correspond to expected length. Received message: {raw_message:x?}"
    )]
    InvalidLength { raw_message: Vec<u8> },
    #[error("Negative response code was received: {nrc:?}")]
    NRC { nrc: NrcData },
    #[error("Was not able to represent provided NRC: {unknown_nrc:x} as the valid NRC")]
    UnknownNRC { rejected_sid: u8, unknown_nrc: u8 },
    #[error("Received message has length of 0")]
    ResponseEmpty,
    #[error("Subfunction {unsupported_subfunction:x} is not supported for used service")]
    UnsupportedSubfunction { unsupported_subfunction: u8 },
    #[error("Argument or combination of entered arguments is not valid")]
    InvalidArgument,
    #[error("something is not correct with received data the data: {raw_message:x?}")]
    ResponseIncorrect { raw_message: Vec<u8> },
    #[error("feature you tried to call is not yet implemented")]
    NotImplemented,
    #[error("Request to be sent is empty")]
    RequestEmpty,
    #[error("Error from lower layer {error:?}")]
    CommunicationError { error: UdsCommunicationError },
}

/// Struct containing rejected sid and nrc for UdsError::Enc type
#[derive(Debug, PartialEq)]
pub struct NrcData {
    pub rejected_sid: u8,
    pub nrc: NegativeResponseCode,
}

impl From<UdsCommunicationError> for UdsError {
    fn from(error: UdsCommunicationError) -> UdsError {
        UdsError::CommunicationError { error }
    }
}

impl From<communication::Error> for UdsError {
    fn from(error: communication::Error) -> UdsError {
        let error: UdsCommunicationError = error.into();
        UdsError::CommunicationError { error }
    }
}

/// Main struct providing all API calls.
pub struct UdsClient {
    socket: UdsSocket,
}

impl UdsClient {
    pub fn new(
        canifc: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
    ) -> Result<UdsClient, UdsError> {
        Ok(UdsClient {
            socket: UdsSocket::new(canifc, src, dst)?,
        })
    }

    pub fn new_vw(
        canifc: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
    ) -> Result<UdsClient, UdsError> {
        Ok(UdsClient {
            socket: UdsSocket::new_vw(canifc, src, dst)?,
        })
    }

    pub fn new_from_socket(socket: UdsSocket) -> UdsClient {
        UdsClient { socket }
    }

    async fn send_and_receive(&self, request: &[u8]) -> Result<Vec<u8>, UdsError> {
        let mut retry_counter = 0;
        if request.is_empty() {
            return Err(UdsError::RequestEmpty);
        }
        self.socket.send(request).await?;
        let mut raw_response = self.socket.receive().await?;

        while let Err(e) = parse_for_error(&raw_response) {
            match e {
                UdsError::NRC { nrc } => {
                    if nrc.rejected_sid != request[0] {
                        return Err(UdsError::SidMismatch {
                            expected: request[0],
                            received: nrc.rejected_sid,
                            raw_message: raw_response,
                        });
                    }
                    match nrc.nrc {
                        NegativeResponseCode::BusyRepeatRequest => {
                            // Maybe sleep a little?
                            retry_counter -= 1;
                            if retry_counter == 0 {
                                warn!("Service failed after multiple repeats");
                                return Err(UdsError::NRC { nrc });
                            }
                            info!("Received NRC BusyRepeatRequest, repeating");
                            self.socket.send(request).await?;
                            raw_response = self.socket.receive().await?;
                        }
                        NegativeResponseCode::RequestCorrectlyReceivedResponsePending => {
                            info!(
                                "NRC RequestCorrectlyReceivedResponsePending received, waiting for next response"
                            );
                            match tokio::time::timeout(
                                Duration::from_millis(2500),
                                self.socket.receive(),
                            )
                            .await
                            {
                                Ok(delayed_response) => {
                                    raw_response = delayed_response?;
                                }
                                Err(_) => {
                                    return Err(UdsError::NRC { nrc });
                                }
                            }
                            break;
                        }
                        _ => return Err(UdsError::NRC { nrc }),
                    }
                }
                _ => {
                    return Err(e);
                }
            }
        }
        Ok(raw_response)
    }
}

fn parse_for_error(raw_response: &[u8]) -> Result<(), UdsError> {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != NEGATIVE_RESPONSE_SID {
        return Ok(());
    }
    let rejected_sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    let nrc: NegativeResponseCode =
        NegativeResponseCode::try_from(*response_iter.next().ok_or(UdsError::ResponseEmpty)?)
            .map_err(|e| UdsError::UnknownNRC {
                rejected_sid,
                unknown_nrc: e.number,
            })?;
    let response = UdsError::NRC {
        nrc: NrcData { rejected_sid, nrc },
    };
    Err(response)
}

#[cfg(test)]
mod tests {
    use crate::uds::uds_definitions::NEGATIVE_RESPONSE_SID;
    use crate::uds::{UdsError, parse_for_error};

    #[test]
    fn test_parse_for_error_wrong_nrc() {
        let raw_response = vec![NEGATIVE_RESPONSE_SID, 0x11, 0xff];
        let expected = UdsError::UnknownNRC {
            rejected_sid: 0x11,
            unknown_nrc: 0xff,
        };
        let result = parse_for_error(&raw_response);
        assert_eq!(Err(expected), result);
    }
}
