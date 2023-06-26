
                    // Some("Low F") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         current_bank = max(0, current_bank - 1);
                    //         synth.program_change(0, current_bank);
                    //         info!("New MIDI bank number {}", current_bank);
                    //         synth.noteon(0, 51, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 51);
                    //     }
                    // }
                    // Some("Low G") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         current_bank = min(128, current_bank + 1);
                    //         synth.program_change(0, current_bank);
                    //         info!("New MIDI bank number {}", current_bank);
                    //         synth.noteon(0, 53, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 53);
                    //     }
                    // }

                    // Some("Low C") => {
                    //     if control_mode {
                    //         control_mode = false;
                    //         info!("Shutting down");
                    //         synth.noteon(0, 46, 127);
                    //         synth.cc(0, MIDI_CC_VOLUME, 127);
                    //         thread::sleep(Duration::from_millis(100));
                    //         synth.noteoff(0, 46);
                    //         shutdown();
                    //     }
                    // }
                    // _ => {
                        // control_command = false;
                    // }