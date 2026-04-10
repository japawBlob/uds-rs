//! Some custom defines for UDS functionality
//!
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SEND_RECEIVE_SID_OFFSET: u8 = 0x40;

pub const NEGATIVE_RESPONSE_SID: u8 = 0x7F;

pub fn to_received_sid(sid: u8) -> u8 {
    sid + SEND_RECEIVE_SID_OFFSET
}

pub fn from_received_sid(sid: u8) -> u8 {
    sid - SEND_RECEIVE_SID_OFFSET
}

/// Used for defining request Service Identifiers (SIDs)
#[derive(IntoPrimitive, TryFromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceIdentifier {
    DiagnosticSessionControl = 0x10,
    EcuReset = 0x11,
    SecurityAccess = 0x27,
    CommunicationControl = 0x28,
    Authentication = 0x29,
    TesterPresent = 0x3E,
    AccessTimingParameters = 0x83,
    SecuredDataTransmission = 0x84,
    ControlDtcSettings = 0x85,
    ResponseOnEvent = 0x86,
    LinkControl = 0x87,
    ReadDataByIdentifier = 0x22,
    ReadMemoryByAddress = 0x23,
    ReadScalingDataByIdentifier = 0x24,
    ReadDataByPeriodicIdentifier = 0x2A,
    DynamicallyDefineDataIdentifier = 0x2C,
    WriteDataByIdentifier = 0x2E,
    WriteMemoryByAddress = 0x3D,
    ClearDiagnosticInformation = 0x14,
    ReadDtcInformation = 0x19,
    InputOutputControlByIdentifier = 0x2F,
    RoutineControl = 0x31,
    RequestDownload = 0x34,
    RequestUpload = 0x35,
    TransferData = 0x36,
    RequestTransferExit = 0x37,
    RequestFileTransfer = 0x38,

    NegativeResponse = 0x7F,
}

/// So called NRC - when server (ECU) sends negative response (SID 0x7F) it is followed by NRC byte, representing the error.
#[derive(IntoPrimitive, TryFromPrimitive, Debug, PartialEq)]
#[repr(u8)]
pub enum NegativeResponseCode {
    PositiveResponse = 0x00,
    GeneralReject = 0x10,
    ServiceNotSupported = 0x11,
    SubFunctionNotSupported = 0x12,
    IncorrectMessageLengthOrInvalidFormat = 0x13,
    ResponseTooLong = 0x14,
    BusyRepeatRequest = 0x21,
    ConditionsNotCorrect = 0x22,
    RequestSequenceError = 0x24,
    NoResponseFromSubnetComponent = 0x25,
    FailurePreventsExecutionOfRequestedAction = 0x26,
    RequestOutOfRange = 0x31,
    SecurityAccessDenied = 0x33,
    InvalidKey = 0x35,
    ExceededNumberOfAttempts = 0x36,
    RequiredTimeDelayNotExpired = 0x37,
    UploadDownloadNotAccepted = 0x70,
    TransferDataSuspended = 0x71,
    GeneralProgrammingFailure = 0x72,
    WrongBlockSequenceCounter = 0x73,
    RequestCorrectlyReceivedResponsePending = 0x78,
    SubfunctionNotSupportedInActiveSession = 0x7E,
    ServiceNotSupportedInActiveSession = 0x7F,
    RpmTooHigh = 0x81,
    RpmTooLow = 0x82,
    EngineIsRunning = 0x83,
    EngineIsNotRunning = 0x84,
    EngineRunTimeTooLow = 0x85,
    TemperatureTooHigh = 0x86,
    TemperatureTooLow = 0x87,
    VehicleSpeedTooHigh = 0x88,
    VehicleSpeedTooLow = 0x89,
    ThrottlePedalTooHigh = 0x8A,
    ThrottlePedalTooLow = 0x8B,
    TransmissionRangeNotInNeutral = 0x8C,
    TransmissionRangeNotInGear = 0x8D,
    BrakeSwitchNotClosed = 0x8F,
    ShiftLeverNotInPark = 0x90,
    TorqueConverterClutchLocked = 0x91,
    VoltageTooHigh = 0x92,
    VoltageTooLow = 0x93,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_received_sid() {
        assert_eq!(0x62, to_received_sid(0x22));
    }

    #[test]
    fn test_from_received_sid() {
        assert_eq!(0x22, from_received_sid(0x62));
    }

    #[test]
    fn test_service_identifier_try_from() {
        assert_eq!(
            Ok(ServiceIdentifier::ReadDataByIdentifier),
            ServiceIdentifier::try_from(0x22)
        );
    }

    #[test]
    fn test_negative_response_code_try_from() {
        assert_eq!(
            Ok(NegativeResponseCode::RequestCorrectlyReceivedResponsePending),
            NegativeResponseCode::try_from(0x78)
        );
    }
}
