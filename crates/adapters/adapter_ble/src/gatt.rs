//! GATT connection helpers for active BLE device readout.
//!
//! Provides [`read_miflora`] which connects to a Mi Flora peripheral,
//! writes the activation command, reads sensor data and firmware info,
//! and always disconnects — even on error.
//!
//! Also provides [`blink_miflora`] which writes the blink LED command
//! to a Mi Flora peripheral.

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;

use crate::error::BleError;
use crate::miflora::{self, CMD_CHAR, DATA_CHAR, FIRMWARE_CHAR, MifloraReading};

/// Command bytes to write to the CMD characteristic to activate sensor mode.
const ACTIVATE_CMD: &[u8] = &[0xa0, 0x1f];

/// Command bytes to write to the CMD characteristic to blink the LED.
const BLINK_CMD: &[u8] = &[0xfd, 0xff];

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
/// The `mac` parameter is the device's real MAC address, typically extracted
/// from the `MiBeacon` `0xFE95` service data advertisement via
/// [`miflora::parse_mibeacon_mac`]. This is necessary because
/// `peripheral.address()` returns a zeroed address on macOS.
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
pub async fn read_miflora(
    peripheral: &Peripheral,
    mac: [u8; 6],
) -> Result<MifloraReading, BleError> {
    peripheral.connect().await.map_err(BleError::GattConnect)?;

    let result = read_miflora_inner(peripheral, mac).await;

    if let Err(err) = peripheral.disconnect().await {
        tracing::warn!(%err, "failed to disconnect Mi Flora peripheral");
    }

    result
}

/// Inner read logic, separated so the caller can always disconnect.
async fn read_miflora_inner(
    peripheral: &Peripheral,
    mac: [u8; 6],
) -> Result<MifloraReading, BleError> {
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

    Ok(MifloraReading {
        mac,
        sensor,
        firmware,
    })
}

/// Connect to a Mi Flora peripheral, write the blink LED command, and
/// disconnect.
///
/// The connection is always closed on return, even if the write fails.
/// The caller is responsible for applying a per-device timeout around
/// this function.
///
/// # Protocol
///
/// 1. Connect to the peripheral
/// 2. Discover services and characteristics
/// 3. Write `[0xfd, 0xff]` to the CMD characteristic (blink LED)
/// 4. Disconnect
///
/// # Errors
///
/// Returns [`BleError::GattConnect`] if the connection fails,
/// [`BleError::CharacteristicNotFound`] if the CMD characteristic is
/// missing, or [`BleError::Scan`] for write failures.
pub async fn blink_miflora(peripheral: &Peripheral) -> Result<(), BleError> {
    peripheral.connect().await.map_err(BleError::GattConnect)?;

    let result = blink_miflora_inner(peripheral).await;

    if let Err(err) = peripheral.disconnect().await {
        tracing::warn!(%err, "failed to disconnect Mi Flora peripheral after blink");
    }

    result
}

/// Inner blink logic, separated so the caller can always disconnect.
async fn blink_miflora_inner(peripheral: &Peripheral) -> Result<(), BleError> {
    peripheral.discover_services().await?;

    let cmd_char = find_characteristic(peripheral, CMD_CHAR)?;

    peripheral
        .write(&cmd_char, BLINK_CMD, WriteType::WithResponse)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_correct_activate_command() {
        assert_eq!(ACTIVATE_CMD, &[0xa0, 0x1f]);
    }

    #[test]
    fn should_have_correct_blink_command() {
        assert_eq!(BLINK_CMD, &[0xfd, 0xff]);
    }
}
