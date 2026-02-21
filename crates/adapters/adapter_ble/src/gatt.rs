//! GATT connection helpers for active BLE device readout.
//!
//! Provides [`read_miflora`] which connects to a Mi Flora peripheral,
//! writes the activation command, reads sensor data and firmware info,
//! and always disconnects — even on error.

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

use crate::error::BleError;
use crate::miflora::{self, CMD_CHAR, DATA_CHAR, FIRMWARE_CHAR, MifloraReading};

/// Command bytes to write to the CMD characteristic to activate sensor mode.
const ACTIVATE_CMD: &[u8] = &[0xa0, 0x1f];

/// Find a GATT characteristic by UUID on a peripheral that has already
/// discovered its services.
///
/// # Errors
///
/// Returns [`BleError::CharacteristicNotFound`] if no characteristic with
/// the given UUID is present.
fn find_characteristic(
    peripheral: &Peripheral,
    uuid: uuid::Uuid,
) -> Result<Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or(BleError::CharacteristicNotFound { uuid })
}

/// Connect to a Mi Flora peripheral, read sensor data and firmware info,
/// and return a [`MifloraReading`].
///
/// The connection is always closed on return, even if a read fails. The
/// caller is responsible for applying a per-device timeout around this
/// function.
///
/// # Protocol
///
/// 1. Connect to the peripheral
/// 2. Discover services and characteristics
/// 3. Write `[0xa0, 0x1f]` to the CMD characteristic (activate sensor mode)
/// 4. Read the 16-byte DATA characteristic
/// 5. Read the 7-byte FIRMWARE characteristic
/// 6. Disconnect
///
/// # Errors
///
/// Returns [`BleError::GattConnect`] if the connection fails,
/// [`BleError::CharacteristicNotFound`] if a required characteristic is
/// missing, or [`BleError::PayloadParse`] / [`BleError::Scan`] for
/// read/write failures.
pub async fn read_miflora(peripheral: &Peripheral) -> Result<MifloraReading, BleError> {
    peripheral.connect().await.map_err(BleError::GattConnect)?;

    let result = read_miflora_inner(peripheral).await;

    if let Err(err) = peripheral.disconnect().await {
        tracing::warn!(%err, "failed to disconnect Mi Flora peripheral");
    }

    result
}

/// Inner read logic, separated so the caller can always disconnect.
async fn read_miflora_inner(peripheral: &Peripheral) -> Result<MifloraReading, BleError> {
    peripheral.discover_services().await?;

    let cmd_char = find_characteristic(peripheral, CMD_CHAR)?;
    let data_char = find_characteristic(peripheral, DATA_CHAR)?;
    let firmware_char = find_characteristic(peripheral, FIRMWARE_CHAR)?;

    peripheral
        .write(&cmd_char, ACTIVATE_CMD, WriteType::WithResponse)
        .await?;

    let data_bytes = peripheral.read(&data_char).await?;
    let firmware_bytes = peripheral.read(&firmware_char).await?;

    let sensor = miflora::parse_sensor_data(&data_bytes)?;
    let firmware = miflora::parse_firmware(&firmware_bytes)?;

    let mac = peripheral.address().into_inner();

    Ok(MifloraReading {
        mac,
        sensor,
        firmware,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_correct_activate_command() {
        assert_eq!(ACTIVATE_CMD, &[0xa0, 0x1f]);
    }
}
