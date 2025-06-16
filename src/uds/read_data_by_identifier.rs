#![allow(dead_code)]

//! # Implementation of ReadDataByIdentifier 0x22 service
//!
//! This service provides following methods for UdsClient:
//!
//! [UdsClient::read_data_by_identifier]
//!
//! Problem with [UdsClient::read_data_by_identifier] is that there can be multiple identifiers and multiple
//! responses in single message. Response needs apriori information about the size of each data_record.
//!
//! Can be solved by only allowing one data identifier per message -> then the data len is
//! arbitrary, implemented in [UdsClient::read_single_data_by_identifier].
//!
//! Other approach is to provide lengths of each data - that is implemented in [UdsClient::read_data_by_identifier_tuple]
//!
//! If [UdsClient::read_data_by_identifier] is used with multiple data identifiers, the unparsed response is returned
//!
use super::*;
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;

const READ_DATA_BY_IDENTIFIER_SID: u8 = 0x22;

/// Response of all read_data_by_identifier methods
#[derive(Debug, PartialEq)]
pub struct ReadDataByIdentifierResponse {
    pub data_records: Vec<DataRecord>,
}

/// Single response entry
#[derive(Debug, PartialEq)]
pub struct DataRecord {
    pub data_identifier: u16,
    pub data: Vec<u8>,
}

impl UdsClient {
    pub async fn read_data_by_identifier(&self, data_identifiers: &[u16]) -> EcuResponseResult {
        if data_identifiers.len() == 1 {
            return self
                .read_single_data_by_identifier(data_identifiers[0])
                .await;
        }
        let request = compose_read_data_by_identifier_request(data_identifiers);
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_read_data_by_identifier_response(&raw_response);
        response
    }
    /// Method takes slice of tuples, first element stands for data identifier and second for
    /// data length. Do not like adding another Struct just for this.
    // TODO maybe not so strict checking is in order - the absence of requested data should only trigger warning
    async fn read_data_by_identifier_tuple(
        &self,
        data_identifiers_and_lengths: &[(u16, u32)],
    ) -> EcuResponseResult {
        let mut data_identifiers = vec![];
        for (i, _) in data_identifiers_and_lengths {
            data_identifiers.push(*i);
        }
        let request: Vec<u8> = compose_read_data_by_identifier_request(&data_identifiers);
        let response = self.send_and_receive(&request).await?;
        let parsed_response =
            parse_read_data_by_identifier_tuple_response(data_identifiers_and_lengths, &response);
        return parsed_response;
    }

    async fn read_single_data_by_identifier(&self, data_identifier: u16) -> EcuResponseResult {
        self.read_data_by_identifier_tuple(&[(data_identifier, u32::MAX)])
            .await
    }
}

fn compose_read_data_by_identifier_request(data_identifiers: &[u16]) -> Vec<u8> {
    let mut request: Vec<u8> = vec![READ_DATA_BY_IDENTIFIER_SID];
    for &i in data_identifiers {
        let msb = (i >> 8) as u8;
        request.push(msb);
        let lsb = i as u8;
        request.push(lsb);
    }
    return request;
}

fn parse_read_data_by_identifier_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let ret = UdsResponse::ReadDataByIdentifier(DataFormat::Raw(raw_response[1..].to_vec()));
    Ok(ret)
}

/// When u32::MAX is passed as data len, it reads the whole message - should be used only with single
/// data identifier.
fn parse_read_data_by_identifier_tuple_response(
    data_identifiers_and_lengths: &[(u16, u32)],
    raw_response: &[u8],
) -> EcuResponseResult {
    let mut response_iterator = raw_response.iter();
    let sid = *response_iterator.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;

    if sid != READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }

    let mut data_records = Vec::new();

    for &(did, len) in data_identifiers_and_lengths {
        let msb = *response_iterator.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
        let lsb = *response_iterator.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
        let response_did = ((msb as u16) << 8) + (lsb as u16);

        if did != response_did {
            return Err(UdsError::DidMismatch {
                expected: did,
                received: response_did,
                raw_message: raw_response.to_vec(),
            });
        }
        let mut data_record: Vec<u8> = Vec::new();
        if len != u32::MAX {
            for _ in 0..len {
                data_record.push(*response_iterator.next().ok_or(UdsError::InvalidLength {
                    raw_message: raw_response.to_vec(),
                })?);
            }
        } else {
            loop {
                let next_element = response_iterator.next();
                match next_element {
                    Some(&x) => data_record.push(x),
                    None => break,
                }
            }
        }

        let data_record = DataRecord {
            data_identifier: response_did,
            data: data_record,
        };
        data_records.push(data_record);
    }
    // Do I care if there are some bytes left in the message? I would say yes and should throw
    // en error (or maybe a warning?)

    let read_data_by_identifier_response = ReadDataByIdentifierResponse { data_records };

    let ret =
        UdsResponse::ReadDataByIdentifier(DataFormat::Parsed(read_data_by_identifier_response));
    return Ok(ret);
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ok_message() {
        let data_identifiers_and_lengths = vec![(10, 1), (20, 10)];
        let dummy_message = vec![
            READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            0,
            10,
            0,
            0,
            20,
            0,
            0,
            0,
            0,
            0,
            1,
            2,
            3,
            4,
            5,
        ];

        let reference =
            UdsResponse::ReadDataByIdentifier(DataFormat::Parsed(ReadDataByIdentifierResponse {
                data_records: vec![
                    DataRecord {
                        data_identifier: 10,
                        data: vec![0],
                    },
                    DataRecord {
                        data_identifier: 20,
                        data: vec![0, 0, 0, 0, 0, 1, 2, 3, 4, 5],
                    },
                ],
            }));

        let result = parse_read_data_by_identifier_tuple_response(
            &data_identifiers_and_lengths,
            &dummy_message,
        );
        assert_eq!(result, Ok(reference));
    }
    #[test]
    fn test_did_mismatch() {
        let data_identifiers_and_lengths = vec![(10, 1), (20, 10)];
        let dummy_message = vec![
            READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            0,
            10,
            0,
            0,
            21,
            0,
            0,
            0,
            0,
            0,
            1,
            2,
            3,
            4,
            5,
        ];

        let result = parse_read_data_by_identifier_tuple_response(
            &data_identifiers_and_lengths,
            &dummy_message,
        );
        assert_eq!(
            result,
            Err(UdsError::DidMismatch {
                expected: 20,
                received: 21,
                raw_message: dummy_message
            })
        );
    }
    #[test]
    fn test_sid_mismatch() {
        let data_identifiers_and_lengths = vec![(10, 1), (20, 10)];
        let dummy_message = vec![
            READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET + 10,
            0,
            10,
            0,
            0,
            20,
            0,
            0,
            0,
            0,
            0,
            1,
            2,
            3,
            4,
            5,
        ];

        let result = parse_read_data_by_identifier_tuple_response(
            &data_identifiers_and_lengths,
            &dummy_message,
        );
        assert_eq!(
            result,
            Err(UdsError::SidMismatch {
                expected: READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
                received: READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET + 10,
                raw_message: dummy_message,
            })
        );
    }
    #[test]
    fn test_invalid_length_short() {
        let data_identifiers_and_lengths = vec![(10, 1), (20, 10), (21, 0)];
        let dummy_message = vec![
            READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            0,
            10,
            0,
            0,
            20,
            0,
            0,
            0,
            0,
            0,
            1,
            2,
            3,
            4,
            0,
            21,
        ];

        let result = parse_read_data_by_identifier_tuple_response(
            &data_identifiers_and_lengths,
            &dummy_message,
        );
        assert_eq!(
            result,
            Err(UdsError::InvalidLength {
                raw_message: dummy_message
            })
        );
    }
    #[test]
    fn test_parse_with_zero_len() {
        let data_identifiers_and_lengths = vec![(10, u32::MAX)];
        let dummy_message = vec![
            READ_DATA_BY_IDENTIFIER_SID + SEND_RECEIVE_SID_OFFSET,
            0,
            10,
            32,
            32,
            21,
            65,
            9,
            8,
            7,
            4,
        ];

        let reference =
            UdsResponse::ReadDataByIdentifier(DataFormat::Parsed(ReadDataByIdentifierResponse {
                data_records: vec![DataRecord {
                    data_identifier: 10,
                    data: vec![32, 32, 21, 65, 9, 8, 7, 4],
                }],
            }));

        let result = parse_read_data_by_identifier_tuple_response(
            &data_identifiers_and_lengths,
            &dummy_message,
        );
        assert_eq!(result, Ok(reference));
    }
    #[test]
    fn test_formulate_request() {
        let data_identifiers = vec![0x10, 0x20, 0x30, 0x4320, 0x4200];
        let result = compose_read_data_by_identifier_request(&data_identifiers);
        let reference = vec![
            READ_DATA_BY_IDENTIFIER_SID,
            0x0,
            0x10,
            0x0,
            0x20,
            0x0,
            0x30,
            0x43,
            0x20,
            0x42,
            0x00,
        ];
        assert_eq!(result, reference);
    }
}
