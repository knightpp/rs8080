extern crate rs8080_emulator as emulator;
#[cfg(feature = "sound")]
use crate::invaders_sound::AudioCircuit;
use emulator::DataBus;

#[cfg(feature = "sound")]
pub(crate) struct SpaceInvadersIO {
    ports: [u8; 6],
    shift0: u8,
    shift1: u8,
    shift_offset: u8,
    audio_circuit: AudioCircuit,
}
#[cfg(not(feature = "sound"))]
pub(crate) struct SpaceInvadersIO {
    ports: [u8; 6],
    shift0: u8,
    shift1: u8,
    shift_offset: u8,
}

impl SpaceInvadersIO {
    #[cfg(feature = "sound")]
    pub fn new() -> SpaceInvadersIO {
        SpaceInvadersIO {
            ports: [0; 6],
            shift0: 0,
            shift1: 0,
            shift_offset: 0,
            audio_circuit: AudioCircuit::new(),
        }
    }

    #[cfg(not(feature = "sound"))]
    pub fn new() -> SpaceInvadersIO {
        SpaceInvadersIO {
            ports: [0; 6],
            shift0: 0,
            shift1: 0,
            shift_offset: 0,
        }
    }

    #[cfg(feature = "sound")]
    pub fn get_audio(&self) -> &AudioCircuit {
        &self.audio_circuit
    }

    fn set_shift_offset(&mut self, offset: u8) {
        self.shift_offset = offset & 0x7;
    }

    fn shift(&self) -> u8 {
        (((self.shift0 as u16) << 8) | self.shift1 as u16).rotate_left(self.shift_offset as u32)
            as u8
    }
}

impl DataBus for SpaceInvadersIO {
    fn port_in(&mut self, port: u8) -> u8 {
        match port {
            0 => 0xf,
            1 => self.ports[1],
            3 => self.shift(),
            _ => self.ports[port as usize],
        }
    }

    fn port_out(&mut self, value: u8, port: u8) {
        match port {
            2 => {
                self.set_shift_offset(value);
            }
            3 => {
                #[cfg(feature = "sound")]
                {
                    if value & 0x1 > 0 && self.ports[3] & 0x1 == 0 {
                        self.audio_circuit.start_playing_ufo_highpitch();
                    } else if value & 0x1 == 0 && self.ports[3] & 0x1 > 0 {
                        self.audio_circuit.stop_playing_ufo_highpitch();
                    }
                    if value & 0x2 > 0 && self.ports[3] & 0x2 == 0 {
                        self.audio_circuit.play_shot();
                    }
                    if value & (1 << 2) > 0 && self.ports[3] & (1 << 2) == 0 {
                        self.audio_circuit.play_player_die();
                    }
                    if value & (1 << 3) > 0 && self.ports[3] & (1 << 3) == 0 {
                        self.audio_circuit.play_invader_die();
                    }
                }
                self.ports[3] = value;
            }
            4 => {
                self.shift0 = self.shift1;
                self.shift1 = value;
            }
            5 => {
                #[cfg(feature = "sound")]
                {
                    if value & (1 << 0) > 0 && self.ports[5] & (1 << 0) == 0 {
                        self.audio_circuit.play_fleet1();
                    }
                    if value & (1 << 1) > 0 && self.ports[5] & (1 << 1) == 0 {
                        self.audio_circuit.play_fleet2();
                    }
                    if value & (1 << 2) > 0 && self.ports[5] & (1 << 2) == 0 {
                        self.audio_circuit.play_fleet3();
                    }
                    if value & (1 << 3) > 0 && self.ports[5] & (1 << 3) == 0 {
                        self.audio_circuit.play_fleet4();
                    }
                }
                self.ports[5] = value;
            }
            _ => {}
        }
    }

    fn port(&mut self, index: usize) -> &mut u8 {
        &mut self.ports[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invaders_shift_register() {
        let mut io = SpaceInvadersIO::new();
        io.port_out(0xFF, 4); // write 0xFF to shift1
        assert_eq!(0xFF, io.shift1);
        io.port_out(0, 2); // set offset to 0
        assert_eq!(io.shift_offset, 0);
        io.port_out(0b0000_0111, 2); // set shift_offset to 7
        assert_eq!(io.shift_offset, 7);
        assert_eq!(io.port_in(3), 0xFF << 7);

        io.port_out(13, 4); // write 13 to shift1, shift0 = 0xFF
        io.port_out(3, 2); // set shift_offset to 3
        assert_eq!(io.port_in(3), (0x0DFF >> (8 - 3)) as u8);
    }
}
