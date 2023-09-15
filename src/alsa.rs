use alsa::card::Iter;

use log::info;

use std::error::Error;

const HAXOPHONE_AUDIO_CARD_ID: &str = "MAX98357A";

pub fn get_device() -> Result<String, Box<dyn Error>> {
    let cards = Iter::new();
    for c in cards {
        let card = c?;
        let id_string = format!("hw:{}", card.get_index());
        let card_name = card.get_name()?;
        info!("Found alsa card {}", card_name);
        if card_name == HAXOPHONE_AUDIO_CARD_ID {
            return Ok(id_string);
        }
    }
    Err("Audio card not found".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use test_log::test;

    #[test]
    fn test_get_device() -> Result<(), Box<dyn Error>> {
        let _ = get_device()?;
        Ok(())
    }
}
