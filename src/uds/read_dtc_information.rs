//! # Implementation of ReadDTCInformation 0x19 service
//!
//! This module provides following methods for UdsClient:
//!
//! [UdsClient::report_number_of_dtc_by_status_mask]  subfunction 0x01  
//! [UdsClient::report_dtc_by_status_mask]  subfunction 0x02  
//! [UdsClient::report_dtc_snapshot_record_by_dtc_number]  subfunction 0x04  
//! [UdsClient::report_number_of_dtc_by_status_mask]  subfunction 0x06  
//! [UdsClient::report_most_recent_confirmed_dtc]  subfunction 0x0e  
//!
use super::*;
use crate::uds::uds_definitions::SEND_RECEIVE_SID_OFFSET;
use num_enum::{IntoPrimitive, TryFromPrimitive};

const READ_DTC_INFORMATION_SID: u8 = 0x19;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, PartialEq)]
pub enum ReadDTCInformationResponse {
    ReportNumberOfDTCbyStatusMask(ReportNumberOfDTCByMaskResponse),
    ReportDTCByStatusMask(ReportDTCsResponse),
    ReportDTCSnapshotIdentification = 0x03,
    ReportDTCSnapshotRecordByDTCNumber(ReportDTCSnapshotRecordByDTCNumber),
    ReportDTCStoredDataByRecordNumber = 0x05,
    ReportDTCExtDataRecordByDTCNumber = 0x06,
    ReportNumberOfDTCBySeverityMaskRecord(ReportNumberOfDTCByMaskResponse),
    ReportDTCBySeverityMaskRecord = 0x08,
    ReportSeverityInformationOfDTC = 0x09,
    ReportSupportedDTC(ReportDTCsResponse),
    ReportFirstTestFailedDTC(ReportDTCsResponse),
    ReportFirstConfirmedDTC(ReportDTCsResponse),
    ReportMostRecentTestFailedDTC(ReportDTCsResponse),
    ReportMostRecentConfirmedDTC(ReportDTCsResponse),
    ReportMirrorMemoryDTCByStatusMask(ReportDTCsResponse),
    ReportMirrorMemoryDTCExtDataRecordByDTCNumber = 0x10,
    ReportNumberOfMirrorMemoryDTCByStatusMask(ReportNumberOfDTCByMaskResponse),
    ReportNumberOfEmissionsOBDDTCByStatusMask(ReportNumberOfDTCByMaskResponse),
    ReportEmissionsOBDDTCByStatusMask(ReportDTCsResponse),
    ReportDTCFaultDetectionCounter = 0x14,
    ReportDTCWithPermanentStatus(ReportDTCsResponse),
    ReportDTCExtDataRecordByRecordNumber = 0x16,
    ReportUserDefMemoryDTCByStatusMask = 0x17,
    ReportUserDefMemoryDTCSnapshotRecordByDTCNumber = 0x18,
    ReportUserDefMemoryDTCExtDataRecordByDTCNumber = 0x19,
    ReportWWHOBDDTCByMaskRecord = 0x42,
    ReportWWHOBDDTCWithPermanentStatus = 0x55,
}

#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive, Debug, PartialEq, Clone, Copy)]
enum SubFunction {
    ReportNumberOfDTCbyStatusMask = 0x01,
    ReportDTCByStatusMask = 0x02,
    ReportDTCSnapshotIdentification = 0x03,
    ReportDTCSnapshotRecordByDTCNumber = 0x04,
    ReportDTCStoredDataByRecordNumber = 0x05,
    ReportDTCExtDataRecordByDTCNumber = 0x06,
    ReportNumberOfDTCBySeverityMaskRecord = 0x07,
    ReportDTCBySeverityMaskRecord = 0x08,
    ReportSeverityInformationOfDTC = 0x09,
    ReportSupportedDTC = 0x0A,
    ReportFirstTestFailedDTC = 0x0B,
    ReportFirstConfirmedDTC = 0x0C,
    ReportMostRecentTestFailedDTC = 0x0D,
    ReportMostRecentConfirmedDTC = 0x0E,
    ReportMirrorMemoryDTCByStatusMask = 0x0F,
    ReportMirrorMemoryDTCExtDataRecordByDTCNumber = 0x10,
    ReportNumberOfMirrorMemoryDTCByStatusMask = 0x11,
    ReportNumberOfEmissionsOBDDTCByStatusMask = 0x12,
    ReportEmissionsOBDDTCByStatusMask = 0x13,
    ReportDTCFaultDetectionCounter = 0x14,
    ReportDTCWithPermanentStatus = 0x15,
    ReportDTCExtDataRecordByRecordNumber = 0x16,
    ReportUserDefMemoryDTCByStatusMask = 0x17,
    ReportUserDefMemoryDTCSnapshotRecordByDTCNumber = 0x18,
    ReportUserDefMemoryDTCExtDataRecordByDTCNumber = 0x19,
    ReportWWHOBDDTCByMaskRecord = 0x42,
    ReportWWHOBDDTCWithPermanentStatus = 0x55,
}

#[derive(IntoPrimitive, TryFromPrimitive, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
#[allow(non_camel_case_types)]
enum DTCFormat {
    SAE_J2012_DA_DTCFormat_00 = 0x00,
    ISO_14229_1_DTCFormat = 0x01,
    SAE_J1939_73_DTCFormat = 0x02,
    ISO_11992_4_DTCFormat = 0x03,
    SAE_J2012_DA_DTCFormat_04 = 0x04,
}

impl UdsClient {
    /// 0x01
    pub async fn report_number_of_dtc_by_status_mask(
        &self,
        dtc_status_mask: u8,
    ) -> EcuResponseResult {
        let request = compose_report_number_of_dtc_by_status_mask_request(
            SubFunction::ReportNumberOfDTCbyStatusMask,
            dtc_status_mask,
        );
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_report_number_of_dtc_by_status_mask_response(&raw_response);
        response
    }

    /// 0x02
    pub async fn report_dtc_by_status_mask(&self, dtc_status_mask: u8) -> EcuResponseResult {
        let request = compose_report_number_of_dtc_by_status_mask_request(
            SubFunction::ReportDTCByStatusMask,
            dtc_status_mask,
        );
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_report_dtcs(&raw_response);
        response
    }

    // /// 0x03
    // #[allow(dead_code)]
    // async fn report_dtc_snapshot_identification(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    /// 0x04
    /// dtc_mask_record is 3 byte value - most significant byte will be dropped.
    /// Needs database to correctly parse the response. Length of snapshotData can't be derived from
    /// plain response
    async fn report_dtc_snapshot_record_by_dtc_number(
        &self,
        dtc_mask_record: u32,
        dtc_snapshot_record_number: u8,
    ) -> EcuResponseResult {
        let request = compose_report_dtc_snapshot_request(
            SubFunction::ReportDTCSnapshotRecordByDTCNumber,
            dtc_mask_record,
            dtc_snapshot_record_number,
        );
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_report_dtc_snapshot_record_by_dtc_number_response(&raw_response);
        response
    }

    // /// 0x05
    // #[allow(dead_code)]
    // async fn report_dtc_stored_data_by_record_number(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    /// 0x06
    pub async fn report_dtc_ext_data_record_by_dtc_number(
        &self,
        dtc_mask_record: u32,
        dtc_ext_data_record_number: u8,
    ) -> EcuResponseResult {
        let request = compose_report_dtc_ext_data_by_dtc_number_request(
            SubFunction::ReportDTCExtDataRecordByDTCNumber,
            dtc_mask_record,
            dtc_ext_data_record_number,
        );
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_report_dtc_ext_data_by_dtc_number_response(&raw_response);
        response
    }

    // /// 0x07
    // #[allow(dead_code)]
    // async fn report_number_of_dtc_by_severity_mask_record(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x08
    // #[allow(dead_code)]
    // async fn report_dtc_by_severity_mask_record(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x09
    // #[allow(dead_code)]
    // async fn report_severity_information_of_dtc(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x0A
    // #[allow(dead_code)]
    // async fn report_supported_dtc(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x0B
    // #[allow(dead_code)]
    // async fn report_first_test_failed_dtc(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x0C
    // #[allow(dead_code)]
    // async fn report_first_confirmed_dtc(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x0D
    // #[allow(dead_code)]
    // async fn report_most_recent_test_failed_dtc(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    /// 0x0E
    pub async fn report_most_recent_confirmed_dtc(&self) -> EcuResponseResult {
        let request = compose_request_short(SubFunction::ReportMostRecentConfirmedDTC);
        let raw_response = self.send_and_receive(&request).await?;
        let response = parse_report_dtcs(&raw_response);
        response
    }

    // /// 0x0F
    // #[allow(dead_code)]
    // async fn report_mirror_memory_dtc_by_status_mask(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x10
    // #[allow(dead_code)]
    // async fn report_mirror_memory_dtc_ext_data_record_by_dtc_number(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x11
    // #[allow(dead_code)]
    // async fn report_number_of_mirror_memory_dtc_by_status_mask(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x12
    // #[allow(dead_code)]
    // async fn report_number_of_emissions_obddtc_by_status_mask(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x13
    // #[allow(dead_code)]
    // async fn report_emissions_obddtc_by_status_mask(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x14
    // #[allow(dead_code)]
    // async fn report_dtc_fault_detection_counter(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x15
    // #[allow(dead_code)]
    // async fn report_dtc_with_permanent_status(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x16
    // #[allow(dead_code)]
    // async fn report_dtc_ext_data_record_by_record_number(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x17
    // #[allow(dead_code)]
    // async fn report_user_def_memory_dtc_by_status_mask(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x18
    // #[allow(dead_code)]
    // async fn report_user_def_memory_dtc_snapshot_record_by_dtc_number(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x19
    // #[allow(dead_code)]
    // async fn report_user_def_memory_dtc_ext_data_record_by_dtc_number(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }

    // /// 0x42
    // #[allow(dead_code)]
    // async fn report_wwhobddtc_by_mask_record(&self) -> EcuResponseResult {
    //     Err(UdsError::NotImplemented)
    // }
}

#[derive(Debug, PartialEq)]
struct DTCSeverityMaskRecord {
    dtc_status_mask: u8,
    dtc_severity_mask: u8,
}

/// Shared between subfunctions 0x01, 0x02, 0x0F, 0x11, 0x12, 0x13
fn compose_report_number_of_dtc_by_status_mask_request(
    subfunction: SubFunction,
    dtc_status_mask: u8,
) -> Vec<u8> {
    vec![READ_DTC_INFORMATION_SID, subfunction as u8, dtc_status_mask]
}

/// Shared between subfunctions 0x01, 0x07, 0x11, 0x12
#[derive(Debug, PartialEq)]
pub struct ReportNumberOfDTCByMaskResponse {
    dtc_status_availability_mask: u8,
    dtc_format_identifier: DTCFormat,
    dtc_count: u16,
}

/// Shared between subfunctions 0x01, 0x07, 0x11, 0x12
fn parse_report_number_of_dtc_by_status_mask_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let report_type: SubFunction =
        SubFunction::try_from(*response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?)
        .map_err(|_| UdsError::ResponseIncorrect {
            raw_message: raw_response.to_vec(),
        })?;
    let dtc_status_availability_mask: u8 =
        *response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
    let dtc_format_identifier_byte = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let dtc_format_identifier =
        DTCFormat::try_from_primitive(dtc_format_identifier_byte).map_err(|_| {
            UdsError::ResponseIncorrect {
                raw_message: raw_response.to_vec(),
            }
        })?;
    let msb = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let lsb = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let dtc_count: u16 = ((msb as u16) << 8) + lsb as u16;

    let parsed = ReportNumberOfDTCByMaskResponse {
        dtc_status_availability_mask,
        dtc_format_identifier,
        dtc_count,
    };

    let response = match report_type {
        SubFunction::ReportNumberOfDTCbyStatusMask => {
            ReadDTCInformationResponse::ReportNumberOfDTCbyStatusMask(parsed)
        }
        SubFunction::ReportNumberOfDTCBySeverityMaskRecord => {
            ReadDTCInformationResponse::ReportNumberOfDTCBySeverityMaskRecord(parsed)
        }
        SubFunction::ReportNumberOfMirrorMemoryDTCByStatusMask => {
            ReadDTCInformationResponse::ReportNumberOfMirrorMemoryDTCByStatusMask(parsed)
        }
        SubFunction::ReportNumberOfEmissionsOBDDTCByStatusMask => {
            ReadDTCInformationResponse::ReportNumberOfEmissionsOBDDTCByStatusMask(parsed)
        }
        _ => return Err(UdsError::InvalidArgument),
    };
    let ret = UdsResponse::ReadDTCInformation(DataFormat::Parsed(response));
    Ok(ret)
}

/// Shared between 0x02, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x13, 0x15
#[derive(Debug, PartialEq)]
pub struct ReportDTCsResponse {
    pub dtc_status_availability_mask: u8,
    pub dtc_and_status_records: Vec<DTCAndStatusRecord>,
}

#[derive(Debug, PartialEq)]
pub struct DTCAndStatusRecord {
    /// dtc has size of 24 bytes, highest byte of u32 is and should be ignored
    pub dtc: u32,
    // TODO each bit in status of DTC has its meaning. It should be represented as different structure, than plain u8
    pub status_of_dtc: u8,
}

/// Shared between 0x02, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x13, 0x15
fn parse_report_dtcs(raw_response: &[u8]) -> EcuResponseResult {
    let mut response_iter = raw_response.iter();
    let sid = *response_iter.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let report_type = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;
    let dtc_status_availability_mask = *response_iter.next().ok_or(UdsError::InvalidLength {
        raw_message: raw_response.to_vec(),
    })?;

    let mut dtc_and_status_records: Vec<DTCAndStatusRecord> = Vec::new();

    while let Some(&high_byte) = response_iter.next() {
        let middle_byte = *response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
        let low_byte = *response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
        let status_of_dtc = *response_iter.next().ok_or(UdsError::InvalidLength {
            raw_message: raw_response.to_vec(),
        })?;
        let dtc = ((high_byte as u32) << 16) + ((middle_byte as u32) << 8) + low_byte as u32;

        dtc_and_status_records.push(DTCAndStatusRecord { dtc, status_of_dtc });
    }

    let parsed = ReportDTCsResponse {
        dtc_status_availability_mask,
        dtc_and_status_records,
    };

    let sub_function =
        SubFunction::try_from(report_type).map_err(|_| UdsError::ResponseIncorrect {
            raw_message: raw_response.to_vec(),
        })?;

    let response = match sub_function {
        SubFunction::ReportDTCByStatusMask => {
            ReadDTCInformationResponse::ReportDTCByStatusMask(parsed)
        }
        SubFunction::ReportSupportedDTC => ReadDTCInformationResponse::ReportSupportedDTC(parsed),
        SubFunction::ReportFirstTestFailedDTC => {
            ReadDTCInformationResponse::ReportFirstTestFailedDTC(parsed)
        }
        SubFunction::ReportFirstConfirmedDTC => {
            ReadDTCInformationResponse::ReportFirstConfirmedDTC(parsed)
        }
        SubFunction::ReportMostRecentTestFailedDTC => {
            ReadDTCInformationResponse::ReportMostRecentTestFailedDTC(parsed)
        }
        SubFunction::ReportMostRecentConfirmedDTC => {
            ReadDTCInformationResponse::ReportMostRecentConfirmedDTC(parsed)
        }
        SubFunction::ReportMirrorMemoryDTCByStatusMask => {
            ReadDTCInformationResponse::ReportMirrorMemoryDTCByStatusMask(parsed)
        }
        SubFunction::ReportEmissionsOBDDTCByStatusMask => {
            ReadDTCInformationResponse::ReportEmissionsOBDDTCByStatusMask(parsed)
        }
        SubFunction::ReportDTCWithPermanentStatus => {
            ReadDTCInformationResponse::ReportDTCWithPermanentStatus(parsed)
        }
        _ => return Err(UdsError::InvalidArgument),
    };

    let ret = UdsResponse::ReadDTCInformation(DataFormat::Parsed(response));

    Ok(ret)
}

/// Shared between 0x03, 0x04
fn compose_report_dtc_snapshot_request(
    sub_function: SubFunction,
    dtc_mask_record: u32,
    dtc_snapshot_record_number: u8,
) -> Vec<u8> {
    vec![
        READ_DTC_INFORMATION_SID,
        sub_function as u8,
        (dtc_mask_record >> 16) as u8,
        (dtc_mask_record >> 8) as u8,
        dtc_mask_record as u8,
        dtc_snapshot_record_number,
    ]
}

/// Used only by 0x04
#[derive(Debug, PartialEq)]
pub struct ReportDTCSnapshotRecordByDTCNumber {
    dtc_and_status_record: DTCAndStatusRecord,
    snapshot_records: Vec<SnapshotRecord>,
}

#[derive(Debug, PartialEq)]
struct SnapshotRecord {
    dtc_snapshot_record_number: u8,
    dtc_snapshot_record_number_of_identifiers: u8,
    dtc_snapshot_record: Vec<SnapshotData>,
}

#[derive(Debug, PartialEq)]
struct SnapshotData {
    data_identifier: u16,
    snapshot_data: Vec<u8>,
}

/// Used only by 0x04
fn parse_report_dtc_snapshot_record_by_dtc_number_response(
    raw_response: &[u8],
) -> EcuResponseResult {
    let mut response = raw_response.iter();
    let sid = *response.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let ret = UdsResponse::ReadDTCInformation(DataFormat::Raw(raw_response[1..].to_vec()));
    Ok(ret)
}

/// Shared between 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x14, 0x15
fn compose_request_short(sub_function: SubFunction) -> Vec<u8> {
    vec![READ_DTC_INFORMATION_SID, sub_function as u8]
}

/// Shared between 0x06, 0x10
fn compose_report_dtc_ext_data_by_dtc_number_request(
    sub_function: SubFunction,
    dtc_mask_record: u32,
    dtc_ext_data_record_number: u8,
) -> Vec<u8> {
    vec![
        READ_DTC_INFORMATION_SID,
        sub_function as u8,
        (dtc_mask_record >> 16) as u8,
        (dtc_mask_record >> 8) as u8,
        dtc_mask_record as u8,
        dtc_ext_data_record_number,
    ]
}

/// shared between 0x06, 0x10
fn parse_report_dtc_ext_data_by_dtc_number_response(raw_response: &[u8]) -> EcuResponseResult {
    let mut response = raw_response.iter();
    let sid = *response.next().ok_or(UdsError::ResponseEmpty)?;
    if sid != READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET {
        return Err(UdsError::SidMismatch {
            expected: READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET,
            received: sid,
            raw_message: raw_response.to_vec(),
        });
    }
    let ret = UdsResponse::ReadDTCInformation(DataFormat::Raw(raw_response[1..].to_vec()));
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test compose for 0x01 - ReportNumberOfDTCbyStatusMask
    #[test]
    fn test_compose_request_0x01() {
        let sub_function: SubFunction = SubFunction::try_from(0x1).unwrap();
        let dtc_status_mask = 0x42;
        let result = compose_report_number_of_dtc_by_status_mask_request(
            SubFunction::ReportNumberOfDTCbyStatusMask,
            dtc_status_mask,
        );
        let expected = vec![
            READ_DTC_INFORMATION_SID,
            sub_function as u8,
            dtc_status_mask,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_0x01() {
        let sid = READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET;
        let report_type = SubFunction::ReportNumberOfDTCbyStatusMask;
        let dtc_status_availability_mask: u8 = 0x18;
        let dtc_format = DTCFormat::ISO_14229_1_DTCFormat;
        let dtc_count: u16 = 0x100f;
        let raw_response: Vec<u8> = vec![
            sid,
            report_type as u8,
            dtc_status_availability_mask,
            dtc_format as u8,
            (dtc_count >> 8) as u8,
            dtc_count as u8,
        ];
        let result = parse_report_number_of_dtc_by_status_mask_response(&raw_response);
        let expected = UdsResponse::ReadDTCInformation(DataFormat::Parsed(
            ReadDTCInformationResponse::ReportNumberOfDTCbyStatusMask(
                ReportNumberOfDTCByMaskResponse {
                    dtc_status_availability_mask,
                    dtc_format_identifier: dtc_format,
                    dtc_count,
                },
            ),
        ));
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_compose_request_0x02() {
        let sub_function = SubFunction::try_from(0x2).unwrap();
        let dtc_status_mask = 0x0;
        let expected = vec![
            READ_DTC_INFORMATION_SID,
            sub_function as u8,
            dtc_status_mask,
        ];
        let result = compose_report_number_of_dtc_by_status_mask_request(
            SubFunction::ReportDTCByStatusMask,
            dtc_status_mask,
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_0x02() {
        let sid = READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET;
        let report_type = SubFunction::try_from(0x2).unwrap();
        let dtc_status_availability_mask: u8 = 0xff;
        let dtc_and_status_record = vec![
            DTCAndStatusRecord {
                dtc: 0x123456,
                status_of_dtc: 0xff,
            },
            DTCAndStatusRecord {
                dtc: 0x42,
                status_of_dtc: 0x0,
            },
            DTCAndStatusRecord {
                dtc: 0x0,
                status_of_dtc: 0xff,
            },
            DTCAndStatusRecord {
                dtc: 0xffffff,
                status_of_dtc: 0xff,
            },
        ];
        let mut raw_response: Vec<u8> = vec![sid, report_type as u8, dtc_status_availability_mask];
        for record in &dtc_and_status_record {
            raw_response.push((record.dtc >> 16) as u8);
            raw_response.push((record.dtc >> 8) as u8);
            raw_response.push(record.dtc as u8);
            raw_response.push(record.status_of_dtc);
        }
        let result = parse_report_dtcs(&raw_response);
        let expected = UdsResponse::ReadDTCInformation(DataFormat::Parsed(
            ReadDTCInformationResponse::ReportDTCByStatusMask(ReportDTCsResponse {
                dtc_status_availability_mask,
                dtc_and_status_records: dtc_and_status_record,
            }),
        ));
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_empty_response_0x02() {
        let sid = READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET;
        let report_type = SubFunction::try_from(0x2).unwrap();
        let dtc_status_availability_mask: u8 = 0xff;
        let dtc_and_status_record: Vec<DTCAndStatusRecord> = vec![];
        let raw_response = vec![sid, report_type as u8, dtc_status_availability_mask];
        let result = parse_report_dtcs(&raw_response);
        let expected = UdsResponse::ReadDTCInformation(DataFormat::Parsed(
            ReadDTCInformationResponse::ReportDTCByStatusMask(ReportDTCsResponse {
                dtc_status_availability_mask,
                dtc_and_status_records: vec![],
            }),
        ));
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_compose_request_iso_0x04() {
        let sid = READ_DTC_INFORMATION_SID;
        let sub_function = SubFunction::try_from(0x4).unwrap();
        let raw_dtc_mask_record: u32 = 0x12345678;
        let dtc_snapshot_record_number: u8 = 0xff;
        let dtc_mask_record = raw_dtc_mask_record;
        let result = compose_report_dtc_snapshot_request(
            sub_function,
            dtc_mask_record,
            dtc_snapshot_record_number,
        );
        let expected = vec![
            sid,
            sub_function as u8,
            (raw_dtc_mask_record >> 16) as u8,
            (raw_dtc_mask_record >> 8) as u8,
            raw_dtc_mask_record as u8,
            dtc_snapshot_record_number,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_0x04() {
        let raw_response = vec![];
        let result = parse_report_dtc_snapshot_record_by_dtc_number_response(&raw_response);
        assert_eq!(Err(UdsError::ResponseEmpty), result);
    }

    #[test]
    fn test_compose_request_iso_0x06() {
        let sid = READ_DTC_INFORMATION_SID;
        let sub_function = SubFunction::try_from(0x4).unwrap();
        let raw_dtc_mask_record: u32 = 0x12345678;
        let dtc_ext_data_record_number: u8 = 0xff;
        let dtc_mask_record = raw_dtc_mask_record;
        let result = compose_report_dtc_snapshot_request(
            sub_function,
            dtc_mask_record,
            dtc_ext_data_record_number,
        );
        let expected = vec![
            sid,
            sub_function as u8,
            (raw_dtc_mask_record >> 16) as u8,
            (raw_dtc_mask_record >> 8) as u8,
            raw_dtc_mask_record as u8,
            dtc_ext_data_record_number,
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_response_0x06() {
        let raw_response = vec![];
        let result = parse_report_dtc_ext_data_by_dtc_number_response(&raw_response);
        assert_eq!(Err(UdsError::ResponseEmpty), result);
    }

    fn test_compose_request_0x0e() {
        let sid = READ_DTC_INFORMATION_SID;
        let subfunction = SubFunction::try_from(0x0e).unwrap();
        let result = compose_request_short(subfunction);
        assert_eq!(vec![sid, 0x0e], result);
    }

    #[test]
    fn test_parse_response_0x0e() {
        let sid = READ_DTC_INFORMATION_SID + SEND_RECEIVE_SID_OFFSET;
        let report_type = SubFunction::try_from(0xe).unwrap();
        let dtc_status_availability_mask: u8 = 0xff;
        let dtc_and_status_record = vec![DTCAndStatusRecord {
            dtc: 0x123456,
            status_of_dtc: 0xff,
        }];
        let mut raw_response: Vec<u8> = vec![sid, report_type as u8, dtc_status_availability_mask];
        for record in &dtc_and_status_record {
            raw_response.push((record.dtc >> 16) as u8);
            raw_response.push((record.dtc >> 8) as u8);
            raw_response.push(record.dtc as u8);
            raw_response.push(record.status_of_dtc);
        }
        let result = parse_report_dtcs(&raw_response);
        let expected = UdsResponse::ReadDTCInformation(DataFormat::Parsed(
            ReadDTCInformationResponse::ReportMostRecentConfirmedDTC(ReportDTCsResponse {
                dtc_status_availability_mask,
                dtc_and_status_records: dtc_and_status_record,
            }),
        ));
        assert_eq!(result, Ok(expected));
    }
}
