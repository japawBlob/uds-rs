//! # Implementation of ClearDTCInformation 0x14 service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::clear_diagnostic_information]
//!
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;
use crate::uds::{EcuResponseResult, UdsClient, UdsError, UdsResponse};
use log::error;

const CLEAR_DIAGNOSTIC_INFORMATION_SID: u8 = 0x14;

impl UdsClient {
    pub async fn clear_diagnostic_information(&self, group_of_dtc: u32) -> EcuResponseResult {
        let request = compose_clear_diagnostic_information_request(group_of_dtc);
        let raw_response = self.send_and_receive(&request).await?;
        parse_clear_diagnostic_information_response(&raw_response)
    }
}

fn compose_clear_diagnostic_information_request(group_of_dtc: u32) -> Vec<u8> {
    vec![
        CLEAR_DIAGNOSTIC_INFORMATION_SID,
        (group_of_dtc >> 16) as u8,
        (group_of_dtc >> 8) as u8,
        group_of_dtc as u8,
    ]
}

fn parse_clear_diagnostic_information_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != CLEAR_DIAGNOSTIC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET {
        error!("Raw response: {:x?}", raw_response);
        return Err(UdsError::SidMismatch {
            expected: CLEAR_DIAGNOSTIC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let result = UdsResponse::ClearDiagnosticInformation;
    Ok(result)
}
