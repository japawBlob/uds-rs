//! # Implementation of EcuReset 0x11 service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::ecu_reset]
//!
use super::*;
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;
use num_enum::{IntoPrimitive, TryFromPrimitive};

const ECU_RESET_SID: u8 = 0x11;

#[derive(IntoPrimitive, TryFromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum ResetType {
    HardReset = 1,
    KeyOffOnReset = 2,
    SoftReset = 3,
    EnableRapidPowerShutDown = 4,
    DisableRapidPowerShutDown = 5,
}

#[derive(Debug, PartialEq)]
pub struct EcuResetResponse {
    pub reset_type: ResetType,
    pub power_down_time: Option<u8>,
}

impl<T: UdsTransport> UdsClient<T> {
    pub async fn ecu_reset(&self, reset_type: ResetType) -> EcuResponseResult {
        let request = compose_ecu_reset_request(reset_type);
        let raw_response = self.send_and_receive(&request).await?;
        parse_ecu_reset_response(&raw_response)
    }
}

fn compose_ecu_reset_request(reset_type: ResetType) -> Vec<u8> {
    vec![ECU_RESET_SID, reset_type as u8]
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose_key_off_on_reset_request() {
        let result = compose_ecu_reset_request(ResetType::KeyOffOnReset);
        assert_eq!(vec![ECU_RESET_SID, 0x02], result);
    }

    #[test]
    fn test_parse_enable_rapid_power_shutdown_response() {
        let raw_response = vec![0x51, 0x04, 0x0A];
        let result = parse_ecu_reset_response(&raw_response);
        assert_eq!(
            Ok(UdsResponse::EcuReset(DataFormat::Parsed(EcuResetResponse {
                reset_type: ResetType::EnableRapidPowerShutDown,
                power_down_time: Some(0x0A),
            }))),
            result
        );
    }

    #[test]
    fn test_parse_unknown_reset_type() {
        let raw_response = vec![0x51, 0xFF];
        let result = parse_ecu_reset_response(&raw_response);
        assert_eq!(
            Err(UdsError::ResponseIncorrect {
                raw_message: raw_response,
            }),
            result
        );
    }
}
