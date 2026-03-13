# Unified Diagnostics Services (UDS) client over CAN bus

[![Maintenance: actively-developed](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)](https://github.com/rust-lang/cargo/issues/4121)
[![VERSION](https://img.shields.io/crates/v/uds-rs.svg)](https://crates.io/crates/uds-rs)
[![Docs.rs](https://docs.rs/uds-rs/badge.svg)](https://docs.rs/uds-rs/)
[![CI Status](https://github.com/japawBlob/uds-rs/workflows/CI/badge.svg)](https://github.com/japawBlob/uds-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)


<!-- The README below is obtained from lib.rs; please update this with: cargo-rdme -->
<!-- DO NOT EDIT TEXT BELOW THIS LINE -->
<!-- cargo-rdme start -->

provides asynchronous UDS communication via socketcan.

### Hierarchy

module __uds__ - top module containing UdsClient trough which all interaction is provided for the user
services used by UdsClient are stored in separate modules - see for example read_data_by_identifier.rs,
where structure of service module is described

module __communication__ - basic communication framework. Purpose of this module is to provide send
and receive functionality for UdsClient.

All communication was designed to be used primarily with ISO 14229-1:2013 definition of UDS.

## Example

For correct behaviour the can interface needs to be setup correctly using following command:
```bash
sudo ip l set dev can0 up type can bitrate 500000
```

```rust
use uds_rs::{UdsClient, UdsError};
use embedded_can::{Id, StandardId};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), UdsError> {
    // Create client
    let c = UdsClient::new("can0", Id::Standard(unsafe { StandardId::new_unchecked(0x774) }), Id::Standard(unsafe { StandardId::new_unchecked(0x70A) }))?;

    // read ecu VIN
    let read_data_result = c.read_data_by_identifier(&[0xf18a]).await;
    match read_data_result {
        Ok(x) => println!("Read data by identifier received {:#x?}", x),
        Err(e) => eprintln!(
            "Read single data by identifier failed with error: {:#x?}",
            e
        ),
    };

    // reading dtc
    let read_dtc_information = c.report_dtc_by_status_mask(0xff).await;
    match read_dtc_information {
        Ok(x) => println!("Read dtc by status mask: {:#x?}", x),
        Err(e) => eprintln!("Clear diagnostic information failed with error: {:#x?}", e),
    }

    // clear all stored dtc
    let clear_dtc_information = c.clear_diagnostic_information(0xffffff).await;
    match clear_dtc_information {
        Ok(x) => println!("{:#x?}", x),
        Err(e) => eprintln!("Clear diagnostic information failed with error: {:#x?}", e),
    };
    Ok(())
}
```
## Notes for development
### Communication architecture
Current communication architecture is strictly bounded request-response together. It would be
much better to have these two interactions separated into queues and adding one producer for writes and one consumer
for reads.

Without this functionality the services like ReadDataByPeriodicIdentifier cannot be implemented.

### Services implementation
each service consists of three steps  
__compose function__ - serializing service method arguments and other needed
data to Vec\<u8\>  
__send and receive__ - passing composed vector as slice to the communication backend and returning raw response  
__parse function__ - parsing received raw response &\[u8\] and serializing it into UdsMessage

## Notes
For the correct behaviour, you need to have Linux kernel with applied patch:
<https://lore.kernel.org/linux-can/20230818114345.142983-1-lukas.magel@posteo.net/#r>

<!-- cargo-rdme end -->
<!-- FROM HERE ON YOU CAN EDIT THE TEXT -->

## License
This project is licensed under the MIT License - see the LICENSE file for details.
