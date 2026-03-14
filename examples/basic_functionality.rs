// To run the example make sure to set-up a CAN interface first!

use embedded_can::StandardId;
use log::{error, info};
use uds_rs::{ResetType, UdsClient, UdsError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), UdsError> {
    env_logger::init();
    // Create client
    let c = UdsClient::new(
        "can0",
        StandardId::new(0x774).expect("Invalid src id"),
        StandardId::new(0x70A).expect("Invalid dst id"),
    )?;

    // read data by identifier (ecu VIN)
    let read_data_result = c.read_data_by_identifier(&[0xf18a]).await;
    match read_data_result {
        Ok(x) => info!("Read data by identifier received {:#x?}", x),
        Err(e) => error!(
            "Read single data by identifier failed with error: {:#x?}",
            e
        ),
    };

    // reading dtc
    let read_dtc_information = c.report_dtc_by_status_mask(0xff).await;
    match read_dtc_information {
        Ok(x) => info!("Read dtc by status mask: {:#x?}", x),
        Err(e) => error!("Read dtc by status mask failed with error: {:#x?}", e),
    }

    // clear all stored dtc
    let clear_dtc_information = c.clear_diagnostic_information(0xffffff).await;
    match clear_dtc_information {
        Ok(x) => info!("{:#x?}", x),
        Err(e) => error!("Clear diagnostic information failed with error: {:#x?}", e),
    };

    // ecu reset
    let ecu_reset_result = c.ecu_reset(ResetType::KeyOffOnReset).await;
    match ecu_reset_result {
        Ok(x) => info!("{:#x?}", x),
        Err(e) => error!("Ecu reset failed with error: {:#x?}", e),
    };

    Ok(())
}
