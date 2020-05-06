use sdl2::mixer;
use sdl2::mixer::{Chunk, LoaderRWops};

pub(crate) struct AudioCircuit {
    shot: Chunk,
    player_die: Chunk,
    invader_die: Chunk,
    fleet1: Chunk,
    fleet2: Chunk,
    fleet3: Chunk,
    fleet4: Chunk,
    ufo_highpitch: Chunk,
}
impl AudioCircuit {
    pub(crate) fn new() -> Self {
        mixer::open_audio(11025, mixer::AUDIO_S8, 1, 256).unwrap();
        mixer::init(mixer::InitFlag::MID).unwrap();
        mixer::allocate_channels(8);

        let shot = include_bytes!("../sounds/shoot.wav");
        let player_die = include_bytes!("../sounds/explosion.wav");
        let invader_die = include_bytes!("../sounds/invaderkilled.wav");
        let fleet1 = include_bytes!("../sounds/fastinvader1.wav");
        let fleet2 = include_bytes!("../sounds/fastinvader2.wav");
        let fleet3 = include_bytes!("../sounds/fastinvader3.wav");
        let fleet4 = include_bytes!("../sounds/fastinvader4.wav");
        //let ufo_lowpitch = include_bytes!("../sounds/ufo_lowpitch.wav");
        let ufo_highpitch = include_bytes!("../sounds/ufo_highpitch.wav");

        let shot = sdl2::rwops::RWops::from_bytes(shot)
            .unwrap()
            .load_wav()
            .unwrap();
        let player_die = sdl2::rwops::RWops::from_bytes(player_die)
            .unwrap()
            .load_wav()
            .unwrap();
        let invader_die = sdl2::rwops::RWops::from_bytes(invader_die)
            .unwrap()
            .load_wav()
            .unwrap();
        let fleet1 = sdl2::rwops::RWops::from_bytes(fleet1)
            .unwrap()
            .load_wav()
            .unwrap();
        let fleet2 = sdl2::rwops::RWops::from_bytes(fleet2)
            .unwrap()
            .load_wav()
            .unwrap();
        let fleet3 = sdl2::rwops::RWops::from_bytes(fleet3)
            .unwrap()
            .load_wav()
            .unwrap();
        let fleet4 = sdl2::rwops::RWops::from_bytes(fleet4)
            .unwrap()
            .load_wav()
            .unwrap();
        let ufo_highpitch = sdl2::rwops::RWops::from_bytes(ufo_highpitch)
            .unwrap()
            .load_wav()
            .unwrap();
        //let ufo_highpitch = sdl2::rwops::RWops::from_bytes(ufo_highpitch).unwrap().load_wav().unwrap();

        AudioCircuit {
            shot,
            player_die,
            invader_die,
            fleet1,
            fleet2,
            fleet3,
            fleet4,
            ufo_highpitch,
        }
    }

    pub(crate) fn set_volume(&self, volume: u8) {
        mixer::Channel::all().set_volume(volume as i32);
    }

    pub(crate) fn start_playing_ufo_highpitch(&self) {
        mixer::Channel(0).play(&self.ufo_highpitch, -1).unwrap();
    }

    pub(crate) fn stop_playing_ufo_highpitch(&self) {
        mixer::Channel(0).pause();
    }

    pub(crate) fn play_shot(&self) {
        mixer::Channel(1).play(&self.shot, 0).unwrap();
    }

    pub(crate) fn play_invader_die(&self) {
        mixer::Channel(2).play(&self.invader_die, 0).unwrap();
    }

    pub(crate) fn play_player_die(&self) {
        mixer::Channel(3).play(&self.player_die, 0).unwrap();
    }
    pub(crate) fn play_fleet1(&self) {
        mixer::Channel(4).play(&self.fleet1, 0).unwrap();
    }
    pub(crate) fn play_fleet2(&self) {
        mixer::Channel(5).play(&self.fleet2, 0).unwrap();
    }
    pub(crate) fn play_fleet3(&self) {
        mixer::Channel(6).play(&self.fleet3, 0).unwrap();
    }
    pub(crate) fn play_fleet4(&self) {
        mixer::Channel(7).play(&self.fleet4, 0).unwrap();
    }
}
impl Drop for AudioCircuit {
    fn drop(&mut self) {
        mixer::close_audio();
    }
}
