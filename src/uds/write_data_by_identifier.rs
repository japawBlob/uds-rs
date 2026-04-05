//! # Implementation of WriteDataByIdentifier 0x2E service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::write_data_by_identifier]
//!

use crate::DataFormat;
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;
use crate::uds::UdsTransport;
use crate::uds::{EcuResponseResult, UdsClient, UdsError, UdsResponse};

const WRITE_DATA_BY_IDENTIFIER_SID: u8 = 0x2E;

#[derive(Debug, PartialEq)]
pub struct WriteDataByIdentifierResponse {
    pub data_identifier: u16,
}
impl<T: UdsTransport> UdsClient<T> {
    pub async fn write_data_by_identifier(
        &self,
        data_identifier: u16,
        data_record: &[u8],
    ) -> EcuResponseResult {
        let request = compose_write_data_by_identifier_request(data_identifier, data_record);
        let raw_response = self.send_and_receive(&request).await?;
        parse_write_data_by_identifier_response(&raw_response)
    }
}

fn compose_write_data_by_identifier_request(data_identifier: u16, data_record: &[u8]) -> Vec<u8> {
    let mut ret = vec![
        WRITE_DATA_BY_IDENTIFIER_SID,
        (data_identifier >> 8) as u8,
        data_identifier as u8,
    ];
    ret.extend_from_slice(data_record);
    ret
}

fn parse_write_data_by_identifier_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != WRITE_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: WRITE_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let msb = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let lsb = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let data_identifier = ((msb as u16) << 8) + lsb as u16;
    let response =
        UdsResponse::WriteDataByIdentifier(DataFormat::Parsed(WriteDataByIdentifierResponse {
            data_identifier,
        }));
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_write_vin_did_request() {
        let result = compose_write_data_by_identifier_request(0xF190, b"TESTVIN123456789");
        let expected = [
            vec![WRITE_DATA_BY_IDENTIFIER_SID, 0xF1, 0x90],
            b"TESTVIN123456789".to_vec(),
        ]
        .concat();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_parse_positive_response() {
        let raw_response = vec![0x6E, 0xF1, 0x90];
        let result = parse_write_data_by_identifier_response(&raw_response);
        assert_eq!(
            Ok(UdsResponse::WriteDataByIdentifier(DataFormat::Parsed(
                WriteDataByIdentifierResponse {
                    data_identifier: 0xF190,
                },
            ))),
            result
        );
    }

    #[test]
    fn test_parse_response_too_short() {
        let raw_response = vec![0x6E, 0xF1];
        let result = parse_write_data_by_identifier_response(&raw_response);
        assert_eq!(
            Err(UdsError::InvalidLength { raw_message: raw_response }),
            result
        );
    }
}
