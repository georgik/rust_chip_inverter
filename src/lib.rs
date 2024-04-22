// Wokwi Custom Chips with Rust
//
// Very rough prototype by Uri Shaked
//
// Look at chipInit() at the bottom, and open Chrome devtools console to see the debugPrint().

use std::ffi::{c_void, CString};

use wokwi_chip_ll::{
    debugPrint, i2cInit, pinInit, pinRead, pinWatch, pinWrite, I2CConfig, PinId, WatchConfig, BOTH, HIGH, INPUT, LOW, OUTPUT
};

struct Chip {
    pin_in: PinId,
    pin_out: PinId,
}

// chipInit() will be called once per chip instance. We use CHIP_VEC to keep track of all the
// instances, and use the user_data pointer to index into CHIP_VEC.
static mut CHIP_VEC: Vec<Chip> = Vec::new();

pub unsafe fn on_pin_change(user_data: *const c_void, _pin: PinId, value: u32) {
    let chip = &CHIP_VEC[user_data as usize];
    if value == HIGH {
        pinWrite(chip.pin_out, LOW);
    } else {
        pinWrite(chip.pin_out, HIGH);
    }
}

pub unsafe fn log_user_data_hex(message: &str, user_data: *const c_void, length: usize) {
    let data_ptr = user_data as *const u8; // Assuming user_data points to an array of u8
    let data_slice = std::slice::from_raw_parts(data_ptr, length); // Cast the pointer to a slice

    let hex_string: Vec<String> = data_slice.iter().map(|byte| format!("{:02X}", byte)).collect();
    debugPrint(CString::new(format!("{} User Data (Hex): {}", message, hex_string.join(" "))).unwrap().into_raw());
}

// Example usage in i2c_connect, assuming you know the length
pub unsafe extern "C" fn i2c_connect(user_data: *const c_void, address: u8, connect: bool) -> bool {
    debugPrint(CString::new("I2C Connect").unwrap().into_raw());
    log_user_data_hex("I2C Connect", user_data, /* length */ 10);
    true
}

// Handle I2C read
pub unsafe extern "C" fn i2c_read(user_data: *const c_void, address: u8, buffer: *mut u8, length: u32) {
    debugPrint(CString::new("I2C Read").unwrap().into_raw());

}

// Handle I2C write
pub unsafe extern "C" fn i2c_write(user_data: *const c_void, address: u8, buffer: *const u8, length: u32) -> bool {
    debugPrint(CString::new("I2C Write").unwrap().into_raw());
    log_user_data_hex("I2C Write", buffer as *const c_void, length as usize);
    true
}

// Handle I2C disconnect
pub unsafe extern "C" fn i2c_disconnect(user_data: *const c_void, address: u8) {
    debugPrint(CString::new("I2C Disconnect").unwrap().into_raw());
    debugPrint(CString::new(format!("Address: {}", address)).unwrap().into_raw());
    debugPrint(CString::new(format!("User Data: {}", user_data as usize)).unwrap().into_raw());
}

#[no_mangle]
pub unsafe extern "C" fn chipInit() {
    debugPrint(CString::new("Initializing GT911").unwrap().into_raw());

    // Configuration for GT911 I2C touch controller
    let mut chip = Chip {
        pin_in: pinInit(CString::new("IN").unwrap().into_raw(), INPUT),
        pin_out: pinInit(CString::new("OUT").unwrap().into_raw(), OUTPUT),
    };
    let mut config = I2CConfig {
        sda: pinInit(CString::new("SDA").unwrap().into_raw(), OUTPUT),
        scl: pinInit(CString::new("SCL").unwrap().into_raw(), OUTPUT),
        user_data: &mut chip as *mut Chip as *mut c_void,
        address: 0x0,
        connect: i2c_connect as *const c_void, // Cast the function pointer to *const c_void
        read: i2c_read as *const c_void, // Cast the function pointer to *const c_void
        write: i2c_write as *const c_void,
        disconnect: i2c_disconnect as *const c_void,
    };
    i2cInit(&config);
    // let chip = Chip {
    //     pin_in: pinInit(CString::new("IN").unwrap().into_raw(), INPUT),
    //     pin_out: pinInit(CString::new("OUT").unwrap().into_raw(), OUTPUT),
    // };
    // CHIP_VEC.push(chip);
    // let chip = CHIP_VEC.last().unwrap();

    // let value = pinRead(chip.pin_in);
    // pinWrite(chip.pin_out, !value);

    // let watch_config = WatchConfig {
    //     user_data: (CHIP_VEC.len() - 1) as *const c_void,
    //     edge: BOTH,
    //     pin_change: on_pin_change as *const c_void,
    // };

    // pinWatch(chip.pin_in, &watch_config);
}
