//! # Implementation of DiagnosticSessionControl 0x10 service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::diagnostic_session_control]
//!
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;
use crate::uds::{EcuResponseResult, UdsClient, UdsError, UdsResponse};
use log::error;

use super::DataFormat;

const DIAGNOSTIC_SESSION_CONTROL_SID: u8 = 0x10;

#[derive(Debug, PartialEq)]
pub struct DiagnosticSessionControlResponse {
    pub session: u8,
    pub p2: u16,
    pub p2_star: u16,
}

impl UdsClient {
    pub async fn diagnostic_session_control(&self, session_id: u8) -> EcuResponseResult {
        let request = compose_diagnostic_session_control_request(session_id);
        let raw_response = self.send_and_receive(&request).await?;
        parse_diagnostic_session_control_response(&raw_response)
    }
}

fn compose_diagnostic_session_control_request(session_id: u8) -> Vec<u8> {
    vec![DIAGNOSTIC_SESSION_CONTROL_SID, session_id]
}

fn parse_diagnostic_session_control_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != DIAGNOSTIC_SESSION_CONTROL_SID + SEND_RECEIVE_SID_OFFSET {
        error!("Raw response: {:x?}", raw_response);
        return Err(UdsError::SidMismatch {
            expected: DIAGNOSTIC_SESSION_CONTROL_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let session = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let p2_hi = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let p2_lo = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let p2s_hi = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let p2s_lo = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let p2 = ((p2_hi as u16) << 8) + p2_lo as u16;
    let p2_star = ((p2s_hi as u16) << 8) + p2s_lo as u16;

    let result = UdsResponse::DiagnosticSessionControl(DataFormat::Parsed(
        DiagnosticSessionControlResponse {
            session,
            p2,
            p2_star,
        },
    ));
    Ok(result)
}
