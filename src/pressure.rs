use std::cmp::min;
use std::error::Error;

use log::{debug, error /* info, warn */};

use rppal::i2c::I2c;

// Pressure sensor I2C address
const ADDR_PRESSURE_SENSOR: u16 = 0x4D;

pub struct Pressure {
    i2c: rppal::i2c::I2c,
    baseline: i32,
}

impl Pressure {
    pub fn init() -> Result<Pressure, Box<dyn Error>> {
        debug!("I2C: Configuring bus ...");

        let maybe_i2c = I2c::new();

        let mut i2c = match maybe_i2c {
            Ok(i2c) => i2c,
            Err(e) => {
                error!("Failed to initialize I2C.  Check raspi-config.");
                return Err(Box::new(e));
            }
        };

        debug!(
            "I2C: Created on bus {} at {} Hz",
            i2c.bus(),
            i2c.clock_speed()?
        );

        // Set the I2C slave address to the device we're communicating with.
        i2c.set_slave_address(ADDR_PRESSURE_SENSOR)?;

        debug!("I2C: slave address set to {}", ADDR_PRESSURE_SENSOR);

        let baseline = Pressure::read_io(&mut i2c)?;

        let sensor = Pressure {
            i2c: i2c,
            baseline: baseline,
        };

        debug!("I2C: baseline set to {}", sensor.baseline);

        Ok(sensor)
    }

    pub fn read(&mut self) -> Result<i32, Box<dyn Error>> {
        let pressure = Pressure::read_io(&mut self.i2c)?;
        /* TODO: implement calibration based on actually measured baseline and MAX */
        Ok(min((pressure - self.baseline) / 1500, 127))
    }

    fn read_io(i2c: &mut rppal::i2c::I2c) -> Result<i32, Box<dyn Error>> {
        let mut reg = [0u8; 3];
        let mut result;
        i2c.read(&mut reg)?;
        result = reg[0] as i32;
        result <<= 8;
        result |= reg[1] as i32;
        result <<= 8;
        result |= reg[2] as i32;
        if (reg[0] & 0x20) >> 5 == 0x1 {
            result = result - 4194304;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    // Import names from outer (for mod tests) scope.
    use super::*;

    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_init() {
        let mut _sensor = Pressure::init().expect("Failed to initialize pressure sensor");
    }

    #[test]
    fn test_read() -> Result<(), Box<dyn Error>> {
        let mut sensor = Pressure::init().expect("Failed to initialize pressure sensor");
        let _pressure = sensor.read()?;
        Ok(())
    }

    /* This test is ignored by default because it expects pressure readings to change over time.
    In order to do that, you might need to blow some air into the tube.

    Run as
    cargo test -- --ignored --show-output
    */
    #[test]
    #[ignore]
    fn test_read_100() -> Result<(), Box<dyn Error>> {
        let mut sensor = Pressure::init().expect("Failed to initialize pressure sensor");
        let mut prev_read = 0;
        let mut pressure_change_detected = false;
        for _ in 0..100 {
            let pressure = sensor.read()?;
            if prev_read == 0 {
                prev_read = pressure;
            }
            const MIN_EXPECTED_VARIATION: i32 = 5;
            if prev_read + MIN_EXPECTED_VARIATION < pressure {
                pressure_change_detected = true;
                println!("prev_read: {}  pressure: {}", prev_read, pressure);
            }
            thread::sleep(Duration::from_millis(50))
        }
        assert!(pressure_change_detected);
        Ok(())
    }
}
