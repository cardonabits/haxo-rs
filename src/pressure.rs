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
        Ok((pressure - self.baseline) / 1500)
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
