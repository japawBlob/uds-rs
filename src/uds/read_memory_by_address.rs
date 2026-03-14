//! # Implementation of ReadMemoryByAddress 0x23 service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::read_memory_by_address]  
//! [UdsClient::read_memory_by_address_simplified]  
//!
//! Main reason for existence of simplified method is address_and_memory_length_format_identifier
//! argument which can be derived from other entered arguments. However it can have some benefit
//! to have this function to choose size manually, that is why simplified version is not the default.

use super::*;

const READ_MEMORY_BY_ADDRESS_SID: u8 = 0x23;
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;

#[derive(Debug, PartialEq)]
pub struct ReadMemoryByAddressResponse {
    pub data_record: Vec<u8>,
}

impl UdsClient {
    /// address_and_memory_length_format_identifier (explained in ISOTP table 152)
    /// is two values encoded in single message - could be split into two separate
    /// values mem_length and address_length. Or create wrapper, that would take two parameters.
    ///
    /// Or could be even more simplified, by accepting just address and mem_size values, but that
    /// would be less flexible
    ///
    /// Takes memory address and byte size encoded in u8 slice. MSB is at position 0.
    /// Examples are shown in ISOTP documentation chapter 10.3.5.2
    pub async fn read_memory_by_address(
        &self,
        address_and_memory_length_format_identifier: u8,
        memory_address: &[u8],
        memory_size: &[u8],
    ) -> EcuResponseResult {
        let request = formulate_request(
            address_and_memory_length_format_identifier,
            memory_address,
            memory_size,
        );
        let response = self.send_and_receive(&request).await?;
        parse_response(&response)
    }
    /// Simplified method, where address_and_memory_length_format_identifier will be assumed from
    /// provided arguments if not specified.
    /// If assumption will take place, the lowest possible size will be used.
    pub async fn read_memory_by_address_simplified(
        &self,
        memory_address: u64,
        memory_size: u64,
        memory_address_len: Option<u8>,
        memory_size_len: Option<u8>,
    ) -> EcuResponseResult {
        let request_arguments = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        )?;

        self.read_memory_by_address(
            request_arguments.0,
            &request_arguments.1,
            &request_arguments.2,
        )
        .await
    }
}

fn convert_from_simple_to_normal(
    memory_address: u64,
    memory_size: u64,
    memory_address_len: Option<u8>,
    memory_size_len: Option<u8>,
) -> Result<(u8, Vec<u8>, Vec<u8>), UdsError> {
    let mut address_encode_bytes = 0;
    let mut size_encode_bytes = 0;

    let mut i = memory_address;
    while i > 0 {
        i >>= 8;
        address_encode_bytes += 1;
    }
    let mut i = memory_size;
    while i > 0 {
        i >>= 8;
        size_encode_bytes += 1;
    }

    if let Some(mut provided_address_len) = memory_address_len {
        if provided_address_len > 0x8 {
            warn!(
                "address_len needs to be lower or equal to 0xf, using 0xf instead of provided value"
            );
            provided_address_len = 0x8;
        }

        if provided_address_len < address_encode_bytes {
            warn!("Provided address_len can not hold provided memory_address");
            return Err(UdsError::InvalidArgument);
        }
        if provided_address_len > address_encode_bytes {
            address_encode_bytes = provided_address_len;
        }
    }
    if let Some(mut provided_memory_size_len) = memory_size_len {
        if provided_memory_size_len > 0x8 {
            warn!(
                "memory_len needs to be lower or equal to 0xf, using 0xf instead of provided value"
            );
            provided_memory_size_len = 0x8;
        }
        if provided_memory_size_len < size_encode_bytes {
            error!("Provided memory_len can not hold provided memory_size");
            return Err(UdsError::InvalidArgument);
        }
        if provided_memory_size_len > size_encode_bytes {
            size_encode_bytes = provided_memory_size_len;
        }
    }
    let address_and_memory_length_format_identifier =
        (size_encode_bytes << 4) + address_encode_bytes;

    // magic to convert [u8; 8] into [u8; encode_size] by dropping first 8-encode_bytes bytes
    let memory_address_bytes =
        memory_address.to_be_bytes().as_slice()[(8 - address_encode_bytes as usize)..].to_vec();
    let memory_size_bytes =
        memory_size.to_be_bytes().as_slice()[(8 - size_encode_bytes as usize)..].to_vec();
    Ok((
        address_and_memory_length_format_identifier,
        memory_address_bytes,
        memory_size_bytes,
    ))
}

fn formulate_request(
    address_and_memory_length_format_identifier: u8,
    memory_address: &[u8],
    memory_size: &[u8],
) -> Vec<u8> {
    let mut request: Vec<u8> = vec![
        READ_MEMORY_BY_ADDRESS_SID,
        address_and_memory_length_format_identifier,
    ];
    request.extend_from_slice(memory_address);
    request.extend_from_slice(memory_size);

    request
}
fn parse_response(raw_response: &[u8]) -> EcuResponseResult {
    let sid = raw_response[0];
    if sid != READ_MEMORY_BY_ADDRESS_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_MEMORY_BY_ADDRESS_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_owned(),
        });
    }
    let read_memory_data = ReadMemoryByAddressResponse {
        data_record: Vec::from(&raw_response[1..]),
    };
    let parsed_response = UdsResponse::ReadMemoryByAddress(DataFormat::Parsed(read_memory_data));
    Ok(parsed_response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_convert_from_simple_to_normal() {
        let memory_address: u64 = 0x12345678;
        let memory_size: u64 = 0x4321;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = None;
        let expected: (u8, Vec<u8>, Vec<u8>) =
            (0x24, vec![0x12, 0x34, 0x56, 0x78], vec![0x43, 0x21]);
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_ok_convert_from_simple_to_normal_specified_memory_address_len() {
        let memory_address: u64 = 0x5678;
        let memory_size: u64 = 0x4321;
        let memory_address_len: Option<u8> = Some(6);
        let memory_size_len: Option<u8> = None;
        let expected: (u8, Vec<u8>, Vec<u8>) =
            (0x26, vec![0x0, 0x0, 0x0, 0x0, 0x56, 0x78], vec![0x43, 0x21]);
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_ok_convert_from_simple_to_normal_specified_memory_size_len() {
        let memory_address: u64 = 0x5678;
        let memory_size: u64 = 0x4321;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = Some(3);
        let expected: (u8, Vec<u8>, Vec<u8>) = (0x32, vec![0x56, 0x78], vec![0x0, 0x43, 0x21]);
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_ok_convert_from_simple_to_normal_specified_both() {
        let memory_address: u64 = 0x5678;
        let memory_size: u64 = 0x4321;
        let memory_address_len: Option<u8> = Some(4);
        let memory_size_len: Option<u8> = Some(3);
        let expected: (u8, Vec<u8>, Vec<u8>) =
            (0x34, vec![0x0, 0x0, 0x56, 0x78], vec![0x0, 0x43, 0x21]);
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_err_convert_from_simple_to_normal_specified_memory_address_len_too_small() {
        let memory_address: u64 = 0x12345678;
        let memory_size: u64 = 0x4321;
        let memory_address_len: Option<u8> = Some(3);
        let memory_size_len: Option<u8> = None;
        let expected = UdsError::InvalidArgument;
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Err(expected));
    }

    #[test]
    fn test_err_convert_from_simple_to_normal_specified_memory_size_len_too_small() {
        let memory_address: u64 = 0x12345678;
        let memory_size: u64 = 0x43211234;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = Some(3);
        let expected = UdsError::InvalidArgument;
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Err(expected));
    }
    #[test]
    fn test_err_convert_from_simple_to_normal_specified_memory_size_len_too_big() {
        let memory_address: u64 = 0x12345678;
        let memory_size: u64 = 0x43211234;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = Some(0xff);
        let expected = (
            0x84,
            vec![0x12, 0x34, 0x56, 0x78],
            vec![0x0, 0x0, 0x0, 0x0, 0x43, 0x21, 0x12, 0x34],
        );
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }
    #[test]
    fn test_err_convert_from_simple_to_normal_specified_memory_address_len_too_big() {
        let memory_address: u64 = 0x12345678;
        let memory_size: u64 = 0x43211234;
        let memory_address_len: Option<u8> = Some(0xff);
        let memory_size_len: Option<u8> = None;
        let expected = (
            0x48,
            vec![0x0, 0x0, 0x0, 0x0, 0x12, 0x34, 0x56, 0x78],
            vec![0x43, 0x21, 0x12, 0x34],
        );
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }
    #[test]
    fn test_err_convert_from_simple_to_normal_provided_address_too_big() {
        let memory_address: u64 = 0x1122_3344_5566_7788;
        let memory_size: u64 = 0x43211234;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = None;
        let expected = (
            0x48,
            vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
            vec![0x43, 0x21, 0x12, 0x34],
        );
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );

        assert_eq!(result, Ok(expected));
    }
    #[test]
    fn test_err_convert_from_simple_to_normal_provided_size_too_big() {
        let memory_address: u64 = 0x1122;
        let memory_size: u64 = 0x1122_3344_5566_7788;
        let memory_address_len: Option<u8> = None;
        let memory_size_len: Option<u8> = None;
        let expected = (
            0x82,
            vec![0x11, 0x22],
            vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
        );
        let result = convert_from_simple_to_normal(
            memory_address,
            memory_size,
            memory_address_len,
            memory_size_len,
        );
        assert_eq!(result, Ok(expected));
    }
    #[test]
    fn test_ok_compose_request() {
        let address_and_memory_length_format_identifier: u8 = 0x24;
        let memory_address: [u8; 4] = [0x4, 0x32, 0x12, 0x1];
        let memory_size: [u8; 2] = [0x1, 0x12];
        let expected = vec![
            READ_MEMORY_BY_ADDRESS_SID,
            0x24,
            0x4,
            0x32,
            0x12,
            0x1,
            0x1,
            0x12,
        ];
        let result = formulate_request(
            address_and_memory_length_format_identifier,
            &memory_address,
            &memory_size,
        );
        assert_eq!(result, expected);
    }
    #[test]
    fn test_ok_parse_response() {
        let sid = READ_MEMORY_BY_ADDRESS_SID + SEND_RECEIVE_SID_OFFSET;
        let data = vec![sid, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let expected =
            UdsResponse::ReadMemoryByAddress(DataFormat::Parsed(ReadMemoryByAddressResponse {
                data_record: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            }));
        let result = parse_response(&data);
        assert_eq!(result, Ok(expected));
    }
}
