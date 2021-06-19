
use std::error::Error;

use rppal::gpio::Gpio;
use rppal::gpio::Level;

// BCM pin numbering
const ROWS: [u8; 8] = [13, 12, 16, 17, 18, 22, 23, 24 ];
const COLS: [u8; 4] = [25 , 26, 27 , 4 ];

pub fn init_io() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new()?;
    for col in &COLS {
        let mut pin = gpio.get(*col)?.into_input_pullup();
        pin.set_reset_on_drop(false);
    }
    for row in &ROWS {
        let mut pin = gpio.get(*row)?.into_output();
        pin.set_high();
        pin.set_reset_on_drop(false);
    }
    Ok(())
}

fn get_bit_at(input: u32, n: u8) -> bool {
    if n < 32 {
        input & (1 << n) != 0
    } else {
        false
    }
}

fn set_bit_at(output: &mut u32, n: u8) {
    if n < 32 {
        *output |= 1 << n;
    }
}

fn clear_bit_at(output: &mut u32, n: u8) {
    if n < 32 {
        *output &= !(1 << n);
    }
}

pub fn scan() -> Result<u32, Box<dyn Error>> {
    const_assert!(ROWS.len() + COLS.len() <= 32);
    let gpio = Gpio::new()?;
    let mut key_idx = 0;
    // a bit if set if the corresponding key is pressed
    let mut keymap: u32 = 0;
    for row in &ROWS {
        let mut row_pin = gpio.get(*row)?.into_output();
        row_pin.set_low();

        for col in &COLS {
            let col_pin = gpio.get(*col)?;
            let is_pressed = col_pin.read() == Level::Low;

            if get_bit_at(keymap, key_idx) != is_pressed {
                if is_pressed {
                    set_bit_at(&mut keymap, key_idx);
                } else {
                    clear_bit_at(&mut keymap, key_idx);
                }
            }
            key_idx += 1;
        }
        row_pin.set_high();
    }
    Ok(keymap)
}

pub fn debug_print(keys: u32) {
    println!("");
    for _col in &COLS {
        print!("==");
    }
    println!("");
    for (i, _col) in COLS.iter().enumerate() {
        print!("{} ", i);
    }
    println!("");
    for _col in &COLS {
        print!("==");
    }
    println!("");
    for (ir, _) in ROWS.iter().enumerate() {
        for (ic, _) in COLS.iter().enumerate() {
            if ic == 0 {
                print!("{}: ", ir);
            }
            let key = get_bit_at(keys, (ir * COLS.len() + ic) as u8);
            print!("{} ", if key { "x" } else { "o" });
        }
        println!("");
    }
}
