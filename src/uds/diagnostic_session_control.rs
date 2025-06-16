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
    session: u8,
    p2: u16,
    p2_star: u16,
}

impl UdsClient {
    pub async fn diagnostic_session_control(&self, session_id: u8) -> EcuResponseResult {
        let request = compose_diagnostic_session_control_request(session_id);
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_diagnostic_session_control_response(&raw_response);
        response
    }
}

fn compose_diagnostic_session_control_request(session_id: u8) -> Vec<u8> {
    vec![
        DIAGNOSTIC_SESSION_CONTROL_SID,
        session_id,
    ]
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

    let result = UdsResponse::DiagnosticSessionControl(DataFormat::Parsed(DiagnosticSessionControlResponse {
        session,
        p2,
        p2_star,
    }));
    Ok(result)
}

/*
fn parse_ecu_reset_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != ECU_RESET_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: ECU_RESET_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let reset_type_byte = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let reset_type: ResetType = ResetType::try_from_primitive(reset_type_byte).map_err(|_| {
        UdsError::ResponseIncorrect {
            raw_message: raw_response.to_vec(),
        }
    })?;
    let mut power_down_time = None;
    if reset_type == ResetType::EnableRapidPowerShutDown {
        power_down_time = Some(*response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?);
    }
    let response = UdsResponse::EcuReset(DataFormat::Parsed(EcuResetResponse {
        reset_type,
        power_down_time,
    }));
    Ok(response)
}
*/