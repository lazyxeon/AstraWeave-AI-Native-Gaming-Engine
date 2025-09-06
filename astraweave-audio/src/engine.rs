use anyhow::{anyhow, Result};
use glam::{vec3, Vec3};
use rodio::{
    source::{SineWave, Source},
    Decoder, OutputStream, OutputStreamHandle, Sink, SpatialSink,
};
use std::{collections::HashMap, fs::File, io::BufReader, time::Duration};

pub type EmitterId = u64;

#[derive(Clone, Copy, Debug)]
pub struct ListenerPose {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
}

/// How to spatialize when using non-spatial SFX.
#[derive(Clone, Copy, Debug)]
pub enum PanMode {
    /// Stereo balance by angle; distance -> volume attenuation.
    StereoAngle,
    /// No spatialization; unity pan.
    None,
}

pub struct MusicTrack {
    pub path: String,
    pub looped: bool,
}

struct MusicChannel {
    a: Sink,
    b: Sink,
    using_a: bool,
    crossfade_time: f32,
    crossfade_left: f32,
    target_vol: f32,
}

impl MusicChannel {
    fn new(handle: &OutputStreamHandle, vol: f32) -> Result<Self> {
        let a = Sink::try_new(handle)?;
        let b = Sink::try_new(handle)?;
        a.set_volume(vol);
        b.set_volume(0.0);
        Ok(Self {
            a,
            b,
            using_a: true,
            crossfade_time: 0.0,
            crossfade_left: 0.0,
            target_vol: vol,
        })
    }

    fn play(
        &mut self,
        handle: &OutputStreamHandle,
        track: &MusicTrack,
        crossfade: f32,
    ) -> Result<()> {
        let file =
            File::open(&track.path).map_err(|e| anyhow!("open music {}: {}", track.path, e))?;
        let src = Decoder::new(BufReader::new(file))?;
        let src: Box<dyn Source<Item = _> + Send> = if track.looped {
            Box::new(src.repeat_infinite())
        } else {
            Box::new(src)
        };

        // start on the inactive sink
        if self.using_a {
            self.b.stop(); // clear previous
            let b = Sink::try_new(handle)?;
            b.set_volume(0.0);
            b.append(src);
            b.play();
            self.b = b;
            self.using_a = false;
        } else {
            self.a.stop();
            let a = Sink::try_new(handle)?;
            a.set_volume(0.0);
            a.append(src);
            a.play();
            self.a = a;
            self.using_a = true;
        }
        self.crossfade_time = crossfade.max(0.01);
        self.crossfade_left = self.crossfade_time;
        Ok(())
    }

    fn set_volume(&mut self, v: f32) {
        self.target_vol = v.max(0.0);
    }

    fn update(&mut self, dt: f32) {
        if self.crossfade_left > 0.0 {
            self.crossfade_left = (self.crossfade_left - dt).max(0.0);
            let k = 1.0 - (self.crossfade_left / self.crossfade_time).clamp(0.0, 1.0);
            let (vol_new, vol_old) = (k * self.target_vol, (1.0 - k) * self.target_vol);
            // apply
            if self.using_a {
                self.a.set_volume(self.target_vol);
                self.b.set_volume(0.0);
            } else {
                self.a.set_volume(vol_old);
                self.b.set_volume(vol_new);
            }
        } else {
            // steady state
            if self.using_a {
                self.a.set_volume(self.target_vol);
                self.b.set_volume(0.0);
            } else {
                self.a.set_volume(self.target_vol);
            }
        }
    }

    fn duck(&mut self, factor: f32) {
        self.target_vol = (self.target_vol * factor).clamp(0.0, 1.0);
        if self.using_a {
            self.a.set_volume(self.target_vol);
        } else {
            self.b.set_volume(self.target_vol);
        }
    }
}

pub struct AudioEngine {
    // core device
    _stream: OutputStream,
    handle: OutputStreamHandle,

    // channels
    music: MusicChannel,
    voice: Sink,
    sfx_bus: Sink, // non-spatial SFX bus

    // spatial sfx per emitter
    spat: HashMap<EmitterId, SpatialSink>,

    // global state
    pub master_volume: f32,
    music_base_volume: f32,
    voice_base_volume: f32,
    sfx_base_volume: f32,

    // listener
    listener: ListenerPose,
    ear_sep: f32, // meters between ears
    pan_mode: PanMode,

    // voice ducking
    duck_timer: f32,
    duck_factor: f32,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let (stream, handle) = OutputStream::try_default()?;

        let music = MusicChannel::new(&handle, 0.8)?;
        let voice = Sink::try_new(&handle)?;
        voice.set_volume(1.0);

        let sfx = Sink::try_new(&handle)?;
        sfx.set_volume(1.0);

        Ok(Self {
            _stream: stream,
            handle,
            music,
            voice,
            sfx_bus: sfx,
            spat: HashMap::new(),
            master_volume: 1.0,
            music_base_volume: 0.8,
            voice_base_volume: 1.0,
            sfx_base_volume: 1.0,
            listener: ListenerPose {
                position: Vec3::ZERO,
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            },
            ear_sep: 0.2,
            pan_mode: PanMode::StereoAngle,
            duck_timer: 0.0,
            duck_factor: 0.4,
        })
    }

    pub fn set_master_volume(&mut self, v: f32) {
        self.master_volume = v.clamp(0.0, 1.0);
        // rodio has no global master; we approximate by scaling channel bases
        let m = self.master_volume;
        self.music.set_volume(self.music_base_volume * m);
        self.voice.set_volume(self.voice_base_volume * m);
        self.sfx_bus.set_volume(self.sfx_base_volume * m);
        for sink in self.spat.values() {
            sink.set_volume(m);
        }
    }

    pub fn set_pan_mode(&mut self, mode: PanMode) {
        self.pan_mode = mode;
    }

    pub fn update_listener(&mut self, pose: ListenerPose) {
        self.listener = pose;
        let ears = self.compute_ears();
        for sink in self.spat.values_mut() {
            sink.set_left_ear_position(ears.0);
            sink.set_right_ear_position(ears.1);
        }
    }

    fn compute_ears(&self) -> ([f32; 3], [f32; 3]) {
        let right = self
            .listener
            .forward
            .cross(self.listener.up)
            .normalize_or_zero();
        let left_pos = self.listener.position - right * (self.ear_sep * 0.5);
        let right_pos = self.listener.position + right * (self.ear_sep * 0.5);
        (left_pos.to_array(), right_pos.to_array())
    }

    pub fn tick(&mut self, dt: f32) {
        // music crossfade & duck restore
        self.music.update(dt);
        if self.duck_timer > 0.0 {
            self.duck_timer -= dt;
            if self.duck_timer <= 0.0 {
                // restore music volume
                self.music
                    .set_volume(self.music_base_volume * self.master_volume);
            }
        }
    }

    pub fn play_music(&mut self, track: MusicTrack, crossfade_sec: f32) -> Result<()> {
        self.music
            .set_volume(self.music_base_volume * self.master_volume);
        self.music.play(&self.handle, &track, crossfade_sec)
    }

    pub fn stop_music(&self) {
        // let sinks finish; not strictly required to stop
        self.music.a.stop();
        self.music.b.stop();
    }

    pub fn play_voice_file(&mut self, path: &str, approximate_sec: Option<f32>) -> Result<()> {
        let file = File::open(path).map_err(|e| anyhow!("open voice {}: {}", path, e))?;
        let src = Decoder::new(BufReader::new(file))?;
        // duck music during voice
        self.music.duck(self.duck_factor);
        if let Some(sec) = approximate_sec {
            self.duck_timer = sec.max(0.1);
        } else {
            // fallback: estimate from samples/ rate if available
            self.duck_timer = 2.5;
        }
        self.voice.append(src);
        self.voice.play();
        Ok(())
    }

    pub fn play_voice_beep(&mut self, text_len: usize) {
        let dur = (text_len as f32 * 0.05).clamp(0.6, 3.0);
        let beep = SineWave::new(600.0)
            .take_duration(Duration::from_secs_f32(dur))
            .amplify(0.2);
        self.music.duck(self.duck_factor);
        self.duck_timer = dur + 0.2;
        self.voice.append(beep);
        self.voice.play();
    }

    pub fn play_sfx_file(&mut self, path: &str) -> Result<()> {
        let file = File::open(path).map_err(|e| anyhow!("open sfx {}: {}", path, e))?;
        let src = Decoder::new(BufReader::new(file))?;
        self.sfx_bus.append(src);
        self.sfx_bus.play();
        Ok(())
    }

    pub fn play_sfx_beep(&mut self, hz: f32, sec: f32, gain: f32) {
        let beep = SineWave::new(hz)
            .take_duration(Duration::from_secs_f32(sec))
            .amplify(gain);
        self.sfx_bus.append(beep);
        self.sfx_bus.play();
    }

    pub fn play_sfx_3d_file(&mut self, emitter: EmitterId, path: &str, pos: Vec3) -> Result<()> {
        let file = File::open(path).map_err(|e| anyhow!("open sfx3d {}: {}", path, e))?;
        let src = Decoder::new(BufReader::new(file))?;
        self.ensure_spatial_sink(emitter)?;
        if let Some(s) = self.spat.get_mut(&emitter) {
            s.set_emitter_position(pos.to_array());
            s.append(src);
            s.play();
        }
        Ok(())
    }

    pub fn play_sfx_3d_beep(
        &mut self,
        emitter: EmitterId,
        pos: Vec3,
        hz: f32,
        sec: f32,
        gain: f32,
    ) -> Result<()> {
        self.ensure_spatial_sink(emitter)?;
        let src = SineWave::new(hz)
            .take_duration(Duration::from_secs_f32(sec))
            .amplify(gain);
        if let Some(s) = self.spat.get_mut(&emitter) {
            s.set_emitter_position(pos.to_array());
            s.append(src);
            s.play();
        }
        Ok(())
    }

    fn ensure_spatial_sink(&mut self, emitter: EmitterId) -> Result<()> {
        if !self.spat.contains_key(&emitter) {
            let (le, re) = self.compute_ears();
            let sink =
                SpatialSink::try_new(&self.handle, le, re, self.listener.position.to_array())?;
            sink.set_volume(self.master_volume);
            self.spat.insert(emitter, sink);
        }
        Ok(())
    }
}
