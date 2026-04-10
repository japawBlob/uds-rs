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
pub struct UdsClient<T: UdsTransport = UdsSocket> {
    socket: T,
}

impl UdsClient<UdsSocket> {
    pub fn new(
        canifc: &str,
        src: impl Into<Id>,
        dst: impl Into<Id>,
        options: UdsSocketOptions,
    ) -> Result<UdsClient<UdsSocket>, UdsError> {
        Ok(UdsClient {
            socket: UdsSocket::new(canifc, src, dst, options)?,
        })
    }
}

impl<T: UdsTransport> UdsClient<T> {
    pub fn new_from_socket(socket: T) -> UdsClient<T> {
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
    use std::collections::VecDeque;
    use std::future::Future;
    use std::sync::Mutex;

    use crate::uds::uds_definitions::NEGATIVE_RESPONSE_SID;
    use crate::uds::{
        DataRecord, DiagnosticSessionControlResponse, EcuResetResponse, NegativeResponseCode,
        ReadDataByIdentifierResponse, ResetType, UdsCommunicationError, UdsError, UdsResponse,
        UdsTransport, parse_for_error,
    };

    use super::{DataFormat, UdsClient};

    struct MockTransport {
        sent_packets: Mutex<Vec<Vec<u8>>>,
        responses: Mutex<VecDeque<Vec<u8>>>,
    }

    impl MockTransport {
        fn new(responses: impl IntoIterator<Item = Vec<u8>>) -> Self {
            Self {
                sent_packets: Mutex::new(Vec::new()),
                responses: Mutex::new(responses.into_iter().collect()),
            }
        }

        fn sent_packets(&self) -> Vec<Vec<u8>> {
            self.sent_packets.lock().unwrap().clone()
        }
    }

    impl UdsTransport for MockTransport {
        fn send(
            &self,
            payload: &[u8],
        ) -> impl Future<Output = Result<(), UdsCommunicationError>> + Send {
            let payload = payload.to_vec();
            async move {
                self.sent_packets.lock().unwrap().push(payload);
                Ok(())
            }
        }

        fn receive(&self) -> impl Future<Output = Result<Vec<u8>, UdsCommunicationError>> + Send {
            async move {
                self.responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .ok_or(UdsCommunicationError::GeneralError)
            }
        }
    }

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

    #[tokio::test]
    async fn test_client_can_be_tested_with_mock_transport() {
        let transport = MockTransport::new([vec![0x51, 0x03]]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.ecu_reset(ResetType::SoftReset).await;

        assert_eq!(
            Ok(UdsResponse::EcuReset(DataFormat::Parsed(
                EcuResetResponse {
                    reset_type: ResetType::SoftReset,
                    power_down_time: None,
                }
            ))),
            result
        );
        assert_eq!(vec![vec![0x11, 0x03]], client.socket.sent_packets());
    }

    #[tokio::test]
    async fn test_busy_repeat_request_retries_send() {
        let transport = MockTransport::new([
            vec![
                NEGATIVE_RESPONSE_SID,
                0x11,
                NegativeResponseCode::BusyRepeatRequest as u8,
            ],
            vec![0x51, 0x03],
        ]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.ecu_reset(ResetType::SoftReset).await;

        assert_eq!(
            Ok(UdsResponse::EcuReset(DataFormat::Parsed(
                EcuResetResponse {
                    reset_type: ResetType::SoftReset,
                    power_down_time: None,
                }
            ))),
            result
        );
        assert_eq!(
            vec![vec![0x11, 0x03], vec![0x11, 0x03]],
            client.socket.sent_packets()
        );
    }

    #[tokio::test]
    async fn test_readme_style_read_data_by_identifier_vin_flow() {
        let transport = MockTransport::new([b"\x62\xF1\x8AWVWZZZ1JZXW00001".to_vec()]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.read_data_by_identifier(&[0xF18A]).await;

        assert_eq!(
            Ok(UdsResponse::ReadDataByIdentifier(DataFormat::Parsed(
                ReadDataByIdentifierResponse {
                    data_records: vec![DataRecord {
                        data_identifier: 0xF18A,
                        data: b"WVWZZZ1JZXW00001".to_vec(),
                    }],
                },
            ))),
            result
        );
        assert_eq!(vec![vec![0x22, 0xF1, 0x8A]], client.socket.sent_packets());
    }

    #[tokio::test]
    async fn test_readme_style_clear_diagnostic_information_flow() {
        let transport = MockTransport::new([vec![0x54]]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.clear_diagnostic_information(0xFF_FF_FF).await;

        assert_eq!(Ok(UdsResponse::ClearDiagnosticInformation), result);
        assert_eq!(
            vec![vec![0x14, 0xFF, 0xFF, 0xFF]],
            client.socket.sent_packets()
        );
    }

    #[tokio::test]
    async fn test_diagnostic_session_control_extended_session_flow() {
        let transport = MockTransport::new([vec![0x50, 0x03, 0x00, 0x32, 0x01, 0xF4]]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.diagnostic_session_control(0x03).await;

        assert_eq!(
            Ok(UdsResponse::DiagnosticSessionControl(DataFormat::Parsed(
                DiagnosticSessionControlResponse {
                    session: 0x03,
                    p2: 0x0032,
                    p2_star: 0x01F4,
                },
            ))),
            result
        );
        assert_eq!(vec![vec![0x10, 0x03]], client.socket.sent_packets());
    }

    #[test]
    fn test_parse_for_error_positive_response_passthrough() {
        assert_eq!(Ok(()), parse_for_error(&[0x50, 0x03]));
    }

    #[test]
    fn test_parse_for_error_empty_response() {
        assert_eq!(Err(UdsError::ResponseEmpty), parse_for_error(&[]));
    }

    #[test]
    fn test_parse_for_error_negative_response_missing_sid() {
        assert_eq!(
            Err(UdsError::ResponseEmpty),
            parse_for_error(&[NEGATIVE_RESPONSE_SID])
        );
    }

    #[test]
    fn test_parse_for_error_negative_response_missing_nrc() {
        assert_eq!(
            Err(UdsError::ResponseEmpty),
            parse_for_error(&[NEGATIVE_RESPONSE_SID, 0x10])
        );
    }

    #[tokio::test]
    async fn test_request_empty_is_rejected_before_transport_use() {
        let transport = MockTransport::new(std::iter::empty::<Vec<u8>>());
        let client = UdsClient::new_from_socket(transport);

        let result = client.send_and_receive(&[]).await;

        assert_eq!(Err(UdsError::RequestEmpty), result);
        assert!(client.socket.sent_packets().is_empty());
    }

    #[tokio::test]
    async fn test_negative_response_with_wrong_rejected_sid_returns_sid_mismatch() {
        let transport = MockTransport::new([vec![
            NEGATIVE_RESPONSE_SID,
            0x22,
            NegativeResponseCode::ConditionsNotCorrect as u8,
        ]]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.ecu_reset(ResetType::SoftReset).await;

        assert_eq!(
            Err(UdsError::SidMismatch {
                expected: 0x11,
                received: 0x22,
                raw_message: vec![
                    NEGATIVE_RESPONSE_SID,
                    0x22,
                    NegativeResponseCode::ConditionsNotCorrect as u8,
                ],
            }),
            result
        );
    }

    #[tokio::test]
    async fn test_response_pending_waits_for_follow_up_response() {
        let transport = MockTransport::new([
            vec![
                NEGATIVE_RESPONSE_SID,
                0x10,
                NegativeResponseCode::RequestCorrectlyReceivedResponsePending as u8,
            ],
            vec![0x50, 0x03, 0x00, 0x32, 0x01, 0xF4],
        ]);
        let client = UdsClient::new_from_socket(transport);

        let result = client.diagnostic_session_control(0x03).await;

        assert_eq!(
            Ok(UdsResponse::DiagnosticSessionControl(DataFormat::Parsed(
                DiagnosticSessionControlResponse {
                    session: 0x03,
                    p2: 0x0032,
                    p2_star: 0x01F4,
                },
            ))),
            result
        );
        assert_eq!(vec![vec![0x10, 0x03]], client.socket.sent_packets());
    }
}
