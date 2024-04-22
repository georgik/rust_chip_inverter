// Wokwi Custom Chips with Rust
//
// Very rough prototype by Uri Shaked
//
// Look at chipInit() at the bottom, and open Chrome devtools console to see the debugPrint().

use std::ffi::{c_void, CString};

use wokwi_chip_ll::{
    debugPrint, i2cInit, pinInit, pinRead, pinWatch, pinWrite, I2CConfig, PinId, WatchConfig, BOTH, HIGH, INPUT, LOW, OUTPUT
};

#[derive(Debug)]
enum ChipState {
    Init,
    Touch
}

struct Chip {
    pin_in: PinId,
    pin_out: PinId,
    chip_state: ChipState,
    init_data: [u8; 3],
    touch_data: [u8; 8], // Touch report data
    current_byte: usize,  // Index of the byte to send next 
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
    let chip = &mut CHIP_VEC[user_data as usize];
    chip.current_byte = 0;
    debugPrint(CString::new(format!("I2C Connect - Address: 0x{:X}", address)).unwrap().into_raw());
    // log_user_data_hex("I2C Connect", user_data, /* length */ 10);
    true
}

// Handle I2C read - Data sent to the microcontroller
pub unsafe extern "C" fn i2c_read(user_data: *const c_void) -> u8 {
    debugPrint(CString::new(format!("I2C Read")).unwrap().into_raw());
    let chip = &mut CHIP_VEC[user_data as usize];

    let byte_to_return = match chip.chip_state {
        ChipState::Init => {
            // Send the initialization data
            let byte_to_return = chip.init_data[chip.current_byte];
            chip.current_byte += 1;
            if chip.current_byte >= chip.init_data.len() {
                chip.current_byte = 0;
                chip.chip_state = ChipState::Touch;
            }
            byte_to_return
        }
        ChipState::Touch => {
            let byte_to_return = chip.touch_data[chip.current_byte];
            chip.current_byte += 1;
            if chip.current_byte >= chip.touch_data.len() {
                chip.current_byte = 0;
            }
            byte_to_return
        }
    };

    // Print chip state and value
    debugPrint(CString::new(format!("Chip State: {:?}, Current Byte: {}", chip.chip_state, byte_to_return)).unwrap().into_raw());    
    // debugPrint(CString::new(format!("Current Byte: {}, value: 0x{:X}", chip.current_byte, byte_to_return)).unwrap().into_raw());


    byte_to_return
}

// Handle I2C write - Data received from the microcontroller
pub unsafe extern "C" fn i2c_write(user_data: *const c_void, buffer: *const u8) -> bool {
    debugPrint(CString::new(format!("I2C Write: 0x{:X}", *buffer)).unwrap().into_raw());
    // log_user_data_hex("I2C Write", buffer as *const c_void, 1);
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
        chip_state: ChipState::Init,
        init_data: [0; 3],
        touch_data: [0; 8],
        current_byte: 0,
    };


    // Calculate middle screen coordinates (assuming 800x1280 resolution)
    let x: i32 = 800 / 2;
    let y: i32 = 1280 / 2;

    // Assuming little-endian byte order
    let x_bytes = x.to_le_bytes();
    let y_bytes = y.to_le_bytes();

    // Update touch data in-place
    chip.touch_data[0] = 0x80;  // Touch down status
    chip.touch_data[1] = 0x01;  // One touch point
    chip.touch_data[2] = x_bytes[1]; // X1 high byte
    chip.touch_data[3] = x_bytes[0]; // X1 low byte
    chip.touch_data[4] = y_bytes[1]; // Y1 high byte
    chip.touch_data[5] = y_bytes[0]; // Y1 low byte

    CHIP_VEC.push(chip);
    let chip = CHIP_VEC.last().unwrap();
    let mut config = I2CConfig {
        sda: pinInit(CString::new("SDA").unwrap().into_raw(), OUTPUT),
        scl: pinInit(CString::new("SCL").unwrap().into_raw(), OUTPUT),
        user_data: (CHIP_VEC.len() - 1) as *const c_void,
        address: 0x5D,
        connect: i2c_connect as *const c_void, // Cast the function pointer to *const c_void
        read: i2c_read as *const c_void, // Cast the function pointer to *const c_void
        write: i2c_write as *const c_void,
        disconnect: i2c_disconnect as *const c_void,
    };
    

    i2cInit(&config);
  
}
