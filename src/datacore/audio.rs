//! `datacore::audio` submodule supplies instruments that help in work with audio data.
//!
//! # Model
//! This submodule consists of structs, traits, enums and constants that can be divided in several groups:
//! 1. Audio data ([`Sound`], [`Music`] and [`Volume`] newtype which encapsulates volume setting).
//! 2. Channels that support audio data ([`SoundChannel`] and [`MusicChannel`], which are both implementors of [`Channel`] trait).
//! 3. Audio system settings ([`AudioSystem::DEFAULT_FREQUENCY`], [`SampleFormat`], [`AudioChannels`], [`AudioSystem::DEFAULT_CHUNK_SIZE`] and
//!    [`AudioSystem`] empty enum which initializes and prepares this submodule for use.
//!
//! To further understand relations between those structs, traits, enums and constants, it is encouraged to read docs for submodule items.
//!

use crate::{
    datacore::assets::{FromFile, ToFile},
    mathcore::Angle,
};
use bitflags::bitflags;
use sdl2::mixer::{
    allocate_channels as mixer_allocate_channels, init as mixer_init,
    open_audio as mixer_open_audio, Channel as MixerChannel, Chunk as MixerChunk,
    InitFlag as MixerInitFlag, Music as MixerMusic, Sdl2MixerContext as MixerContext,
    AUDIO_F32LSB as MixerAUDIO_F32LSB, AUDIO_F32MSB as MixerAUDIO_F32MSB,
    AUDIO_S16LSB as MixerAUDIO_S16LSB, AUDIO_S16MSB as MixerAUDIO_S16MSB,
    AUDIO_S32LSB as MixerAUDIO_S32LSB, AUDIO_S32MSB as MixerAUDIO_S32MSB,
    AUDIO_U16LSB as MixerAUDIO_U16LSB, AUDIO_U16MSB as MixerAUDIO_U16MSB,
    DEFAULT_FREQUENCY as MixerDEFAULT_FREQUENCY, MAX_VOLUME as MixerMAX_VOLUME,
};
use std::{
    fmt,
    io::{Error, ErrorKind},
    num::TryFromIntError,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// [`Volume`] is a newtype that restricts volume values to [0; 128].
///
#[derive(Copy, Clone, Debug, Default)]
pub struct Volume(u8);
impl Volume {
    /// Minimal volume (silence).
    ///
    pub const SILENCE: Self = Volume(0);
    /// Maximal volume.
    ///
    pub const MAX: Self = Volume(MixerMAX_VOLUME as u8);

    /// Returns corresponding `u8` value.
    ///
    pub fn get(self) -> u8 {
        self.0
    }
}
impl From<u8> for Volume {
    fn from(value: u8) -> Self {
        Volume(value.clamp(u8::MIN, Self::MAX.0))
    }
}

/// [`SoundFormat`] trait is implemented on types that are used to represent raw audio data.
///
/// **Do not** implement this trait manually unless a very good reason.
///
pub use sdl2::audio::AudioFormatNum as SoundFormat;

/// [`Sound`] struct represents one of two main audio primitives - short sample.
///
/// Samples (chunks) are meant to be a file completely decoded into memory up front and then be played repeatedly.
/// They might take more memory when initializing, but once they are loaded won't need to decode again.
///
pub struct Sound {
    /// Name of a loaded sound file (`PathBuf` is empty only if sound was manually created from raw buffer).
    ///
    filename: PathBuf,
    /// Underlying `sdl2` chunk.
    ///
    chunk: MixerChunk,
}
impl Sound {
    /// Returns name of file from which [`Sound`] was initialized or empty `Path`, if it was created from raw buffer.
    ///
    pub fn filename(&self) -> &Path {
        self.filename.as_path()
    }

    /// Initializes [`Sound`] from given buffer in specific format.
    ///
    /// No additional conversions will be made.
    ///
    pub fn from_raw_buffer(buffer: Box<[impl SoundFormat]>) -> Result<Self, Error> {
        Ok(Sound {
            filename: PathBuf::new(),
            chunk: MixerChunk::from_raw_buffer(buffer)
                .map_err(|message| Error::new(ErrorKind::InvalidData, message))?,
        })
    }

    /// Sets new volume to sound.
    ///
    pub fn set_volume(&mut self, volume: Volume) {
        let _ = self.chunk.set_volume(volume.get() as i32);
    }
    /// Returns current volume of sound.
    ///
    pub fn get_volume(&self) -> Volume {
        Volume(self.chunk.get_volume() as u8)
    }
}
impl FromFile for Sound {
    /// Initializes [`Sound`] from given file.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::audio::Sound;
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let sound: Sound = Sound::from_file(Path::new("s.wav")).expect("Filename should be correct");
    /// ```
    ///
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Sound {
            filename: path.as_ref().to_path_buf(),
            chunk: MixerChunk::from_file(path)
                .map_err(|message| Error::new(ErrorKind::NotFound, message))?,
        })
    }
}
impl ToFile for Sound {
    /// This is a no-op since all sounds are stored externally.
    ///
    fn to_file(&self, _filename: impl AsRef<Path>) -> Result<(), Error> {
        Ok(())
    }
}
impl fmt::Debug for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sound")
            .field("filename", &self.filename)
            .finish()
    }
}

/// [`Music`] struct represents one of two main audio primitives - long music.
///
/// Music is meant to be a compressed file which is fairly big and thus gets decoded on fly while being played as a background music.
/// Due to absence of predecoding, music does not require as much memory at initialization.
///
pub struct Music {
    /// Name of a loaded music file (`None` only if sound was manually created from raw buffer).
    ///
    filename: PathBuf,
    /// Underlying sdl music.
    ///
    music: MixerMusic<'static>,
}
impl Music {
    /// Returns name of file from which [`Music`] was initialized or empty `Path`, if it was created from raw buffer.
    ///
    pub fn filename(&self) -> &Path {
        self.filename.as_path()
    }

    /// Initializes [`Music`] from given buffer which will be leaked to acquire static reference.
    ///
    /// This function attempts to guess the file format from incoming data.
    ///
    pub fn from_raw_buffer(buffer: Box<[u8]>) -> Result<Self, Error> {
        Ok(Music {
            filename: PathBuf::new(),
            music: MixerMusic::from_static_bytes(Box::leak::<'static>(buffer))
                .map_err(|message| Error::new(ErrorKind::InvalidData, message))?,
        })
    }

    /// Sets new volume to music.
    ///
    pub fn set_volume(&mut self, volume: Volume) {
        MixerMusic::set_volume(volume.get() as i32);
    }
    /// Returns current volume of music.
    ///
    pub fn get_volume(&self) -> Volume {
        Volume(MixerMusic::get_volume() as u8)
    }
}
impl FromFile for Music {
    /// Initializes [`Music`] from given file.
    ///
    /// # Example
    /// ```rust, no_run
    /// # use ggengine::datacore::audio::Music;
    /// # use ggengine::datacore::assets::FromFile;
    /// # use std::path::Path;
    /// let sound: Music = Music::from_file(Path::new("m.mp3")).expect("Filename should be correct");
    /// ```
    ///
    fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Music {
            filename: path.as_ref().to_path_buf(),
            music: MixerMusic::from_file(path)
                .map_err(|message| Error::new(ErrorKind::NotFound, message))?,
        })
    }
}
impl ToFile for Music {
    /// This is a no-op since all sounds are stored externally.
    ///
    fn to_file(&self, _filename: impl AsRef<Path>) -> Result<(), Error> {
        Ok(())
    }
}
impl fmt::Debug for Music {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Music")
            .field("filename", &self.filename)
            .finish()
    }
}

/// [`Channel`] trait defines interface of a channel that supports playing audio data.
///
pub trait Channel {
    /// Type of audio data which channel is supporting.
    ///
    type AudioData;

    /// Plays audio data looping `loops` times.
    ///
    /// For `loops = Some(n)` it will play total of `n + 1` times, for `loops = None` it will be looping infinitely.
    ///
    fn play(&self, data: &Self::AudioData, loops: Option<i32>);
    /// Plays audio data fading in through `fading_time` milliseconds and looping `loops` times.
    ///
    /// For `loops = Some(n)` it will play total of `n + 1` times, for `loops = None` it will be looping infinitely.
    ///
    fn fade_in(&self, data: &Self::AudioData, loops: Option<i32>, fading_time: i32);

    /// Pauses channel.
    ///
    fn pause(&self);
    /// Returns whether the channel is paused or not.
    ///
    fn is_paused(&self) -> bool;
    /// Resumes channel.
    ///
    fn resume(&self);
    /// Returns whether the channel is playing or not.
    ///
    fn is_playing(&self) -> bool;

    /// Stops playing (halts channel).
    ///
    fn stop(&self);
    /// Stops playing by fading out through `fading_time` milliseconds.
    ///
    fn fade_out(&self, fading_time: i32);
}

/// [`SoundChannel`] struct represents channel on which [`Sound`] can be played.
///
/// `ggengine::datacore::audio` supports as many sound channels, as application can allocate.
///
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct SoundChannel(MixerChannel);
impl Channel for SoundChannel {
    type AudioData = Sound;

    fn play(&self, data: &Self::AudioData, loops: Option<i32>) {
        let _ = self
            .0
            .play(&data.chunk, loops.unwrap_or(-1))
            .expect("Audio driver must be available");
    }
    fn fade_in(&self, data: &Self::AudioData, loops: Option<i32>, fading_time: i32) {
        let _ = self
            .0
            .fade_in(&data.chunk, loops.unwrap_or(-1), fading_time)
            .expect("Audio driver must be available");
    }

    fn pause(&self) {
        self.0.pause();
    }
    fn is_paused(&self) -> bool {
        self.0.is_paused()
    }
    fn resume(&self) {
        self.0.resume();
    }
    fn is_playing(&self) -> bool {
        self.0.is_playing()
    }

    fn stop(&self) {
        self.0.halt();
    }
    fn fade_out(&self, fading_time: i32) {
        let _ = self.0.fade_out(fading_time);
    }
}
impl SoundChannel {
    /// Constant that contains all existing channels.
    ///
    pub const ALL: SoundChannel = SoundChannel(MixerChannel(-1));

    /// Returns id of a channel.
    ///
    pub fn id(&self) -> i32 {
        self.0 .0
    }

    /// Constructs [`SoundChannel`] from given id.
    ///
    /// `None` is returned when channel id is incorrect (it exceeds `i32::MAX` or it exceeds the number of channels being allocated).
    ///
    pub fn from_id(id: u32) -> Option<Self> {
        let id: i32 = match i32::try_from(id) {
            Ok(value) => value,
            Err(_) => return None,
        };

        if id >= mixer_allocate_channels(-1) {
            None
        } else {
            Some(SoundChannel(MixerChannel(id)))
        }
    }

    /// If `flip = true`, swaps left and right channel sound. If `flip = false`, effect is unregistered.
    ///
    pub fn reverse_stereo(&self, flip: bool) {
        self.0
            .set_reverse_stereo(flip)
            .expect("Audio driver must be available");
    }

    /// Sets a panning effect, where left and right is the volume of the left and right channels.
    ///
    /// `left` and `right` range from 0 (silence) to 255 (loud).
    ///
    pub fn set_panning(&self, left: u8, right: u8) {
        self.0
            .set_panning(left, right)
            .expect("Audio driver must be available");
    }
    /// This effect simulates a simple attenuation of volume due to distance.
    ///
    /// `distance` ranges from 0 (close/loud) to 255 (far/quiet).
    ///
    pub fn set_distance(&self, distance: u8) {
        self.0
            .set_distance(distance)
            .expect("Audio driver must be available");
    }
    /// This effect emulates a simple 3D audio effect.
    ///
    /// `angle` ranges from 0 to 360 degrees going clockwise, where 0 is directly in front.
    /// `distance` ranges from 0 (close/loud) to 255 (far/quiet).
    ///
    pub fn set_position(&self, angle: Angle, distance: u8) {
        self.0
            .set_position(angle.degrees() as i16, distance)
            .expect("Audio driver must be available");
    }

    /// Unregisters panning effect.
    ///
    pub fn unset_panning(&self) {
        self.0
            .unset_panning()
            .expect("Audio driver must be available");
    }
    /// Unregisters distance effect.
    ///
    pub fn unset_distance(&self) {
        self.0
            .unset_distance()
            .expect("Audio driver must be available");
    }
    /// Unregisters position effect.
    ///
    pub fn unset_position(&self) {
        self.0
            .unset_position()
            .expect("Audio driver must be available");
    }
}
/// [`MusicChannel`] is a singleton that represents channel on which [`Music`] can be played.
///
/// `ggengine::datacore::audio` supports only one channel for playing background music.
///
#[derive(Copy, Clone, Debug)]
pub struct MusicChannel;
impl Channel for MusicChannel {
    type AudioData = Music;

    fn play(&self, data: &Self::AudioData, loops: Option<i32>) {
        data.music
            .play(loops.unwrap_or(-1))
            .expect("Audio driver must be available");
    }
    fn fade_in(&self, data: &Self::AudioData, loops: Option<i32>, fading_time: i32) {
        data.music
            .fade_in(loops.unwrap_or(-1), fading_time)
            .expect("Audio driver must be available");
    }

    fn pause(&self) {
        MixerMusic::pause();
    }
    fn is_paused(&self) -> bool {
        MixerMusic::is_paused()
    }
    fn resume(&self) {
        MixerMusic::resume();
    }
    fn is_playing(&self) -> bool {
        MixerMusic::is_playing()
    }

    fn stop(&self) {
        MixerMusic::halt();
    }
    fn fade_out(&self, fading_time: i32) {
        MixerMusic::fade_out(fading_time).expect("Audio driver must be available");
    }
}

bitflags! (
    /// [`AudioFormat`] bitflag struct lists supported audio formats.
    ///
    pub struct AudioFormat : u32 {
        /// FLAC format flag.
        ///
        const FLAC = 1 << 0;
        /// MOD format flag.
        ///
        const MOD = 1 << 1;
        /// MP3 format flag.
        ///
        const MP3 = 1 << 3;
        /// OGG format flag.
        ///
        const OGG = 1 << 4;
        /// MID format flag.
        ///
        const MID = 1 << 5;
        /// OPUS format flag.
        ///
        const OPUS = 1 << 6;
    }
);

/// [`SampleFormat`] enum lists possible audio formats that are used for decoding samples.
///
/// Abbreviations:
/// 1. `F` stands for float, `S` for signed and `U` for unsigned.
/// 2. `8`/`16`/`32` - how many bits are used in sample size.
/// 3. `LSB` stands for least significant byte (low-order byte, little-endian order),
///    `MSB` for most significant byte (high-order byte, big-endian order),
///    `SYS` for native byte order (those are implemented as constants that depend on target platform).
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SampleFormat {
    /// 32-bit floating point samples in little-endian byte order.
    ///
    F32LSB,
    /// 32-bit floating point samples in big-endian byte order.
    ///
    F32MSB,

    /// Signed 16-bit samples in little-endian byte order.
    ///
    S16LSB,
    /// Signed 16-bit samples in big-endian byte order.
    ///
    S16MSB,
    /// Signed 32-bit samples in little-endian byte order.
    ///
    S32LSB,
    /// Signed 32-bit samples in big-endian byte order.
    ///
    S32MSB,

    /// Unsigned 16-bit samples in little-endian byte order.
    ///
    U16LSB,
    /// Unsigned 16-bit samples in big-endian byte order.
    ///
    U16MSB,
}
#[cfg(target_endian = "little")]
impl SampleFormat {
    /// 32-bit floating point samples in native byte order.
    ///
    pub const F32SYS: Self = Self::F32LSB;
    /// Signed 16-bit samples in native byte order.
    ///
    pub const S16SYS: Self = Self::S16LSB;
    /// Signed 32-bit samples in native byte order.
    ///
    pub const S32SYS: Self = Self::S32LSB;
    /// Unsigned 16-bit samples in native byte order.
    ///
    pub const U16SYS: Self = Self::U16LSB;
}
#[cfg(target_endian = "big")]
impl SampleFormat {
    /// 32-bit floating point samples in native byte order.
    ///
    pub const F32SYS: Self = Self::F32MSB;
    /// Signed 16-bit samples in native byte order.
    ///
    pub const S16SYS: Self = Self::S16MSB;
    /// Signed 32-bit samples in native byte order.
    ///
    pub const S32SYS: Self = Self::S32MSB;
    /// Unsigned 16-bit samples in native byte order.
    ///
    pub const U16SYS: Self = Self::U16MSB;
}
impl SampleFormat {
    // All functions that are providing gate between `ggengine` and `sdl2` extend their API to `crate` visibility.
    /// Returns `sdl2::mixer` representation of [`SampleFormat`] enum.
    ///
    pub(crate) fn to_sdl_u16(self) -> u16 {
        match self {
            SampleFormat::F32LSB => MixerAUDIO_F32LSB,
            SampleFormat::F32MSB => MixerAUDIO_F32MSB,

            SampleFormat::S16LSB => MixerAUDIO_S16LSB,
            SampleFormat::S16MSB => MixerAUDIO_S16MSB,
            SampleFormat::S32LSB => MixerAUDIO_S32LSB,
            SampleFormat::S32MSB => MixerAUDIO_S32MSB,

            SampleFormat::U16LSB => MixerAUDIO_U16LSB,
            SampleFormat::U16MSB => MixerAUDIO_U16MSB,
        }
    }
}
impl Default for SampleFormat {
    fn default() -> Self {
        Self::S32SYS
    }
}
/// [`AudioChannels`] enum lists number of channels that can be used (1 is mono, 2 is stereo, etc.).
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AudioChannels {
    /// Mono (only 1 channel).
    ///
    Mono = 1,
    /// Stereo (2 channels).
    ///
    Stereo = 2,
    /// Quad (4 channels).
    ///
    Quad = 4,
    /// 5.1 (6 channels).
    ///
    FiveOne = 6,
    /// 6.1 (7 channels).
    ///
    SixOne = 7,
    /// 7.1 (8 channels).
    ///
    SevenOne = 8,
}
impl Default for AudioChannels {
    fn default() -> Self {
        Self::Stereo
    }
}
/// [`MIXER_CONTEXT`] global static variable handles `sdl2::mixer` context.
///
static MIXER_CONTEXT: OnceLock<MixerContext> = OnceLock::new();
/// [`AudioSystem`] is a global handler for audio metadata.
///
/// ### `AudioSystem::init` should be called before using anything else from this submodule.
///
#[derive(Copy, Clone, Debug)]
pub enum AudioSystem {}
impl AudioSystem {
    /// Default frequency at which audio is played (Hz).
    ///
    pub const DEFAULT_FREQUENCY: u32 = MixerDEFAULT_FREQUENCY as u32;
    /// Default chunk size (middle ground between latency and compatibility).
    ///
    pub const DEFAULT_CHUNK_SIZE: u32 = 512;

    /// Initializes audio system, prepares libraries for use and opens default audio device for playback.
    ///
    /// Arguments:
    /// 1. `flags` specify audio formats that are going to be supported by the app.
    /// 2. `frequency` is the frequency to playback audio at (in Hz). Recommended value is provided as [`AudioSystem::DEFAULT_FREQUENCY`] const.
    /// 3. `format` is a sample format that will be used. Recommended value is provided as `SampleFormat::default()`.
    /// 4. `channels` is a number of channels (1 is mono, 2 is stereo, etc.). Recommended value is provided as `AudioChannels::default()`.
    /// 5. `chunk_size` is audio buffer size in sample frames (total samples divided by channel count).
    ///    It is recommended to choose values between 256 and 1024, depending on whether you prefer latency or compatibility.
    ///    Small values reduce latency but may not work very well on older systems.
    ///    For instance, a chunk size of 256 will give you a latency of 6ms, while a chunk size of 1024 will give you a latency of 23ms for a frequency of 44100kHz.
    ///    Recommended value is provided as [`AudioSystem::DEFAULT_CHUNK_SIZE`] const.
    ///
    /// # Panics
    /// This function panics when `frequency` or `chunk_size` exceed `i32::MAX`.
    ///
    /// ### `AudioSystem::init` should be called before using anything else from `ggengine::datacore::audio` submodule.
    ///
    pub fn init(
        audio_format: AudioFormat,
        frequency: u32,
        sample_format: SampleFormat,
        channels: AudioChannels,
        chunk_size: u32,
    ) {
        if MIXER_CONTEXT
            .set(
                mixer_init(MixerInitFlag::from_bits(audio_format.bits()).expect(
                    "`AudioFormat` constants are the same as in `InitFlag` bitflags struct",
                ))
                .expect("Audio driver should be available"),
            )
            .is_err()
        {
            return;
        }
        mixer_open_audio(
            i32::try_from(frequency).expect("Frequency value should not exceed `i32::MAX`"),
            sample_format.to_sdl_u16(),
            channels as i32,
            i32::try_from(chunk_size).expect("Chunk size value should not exceed `i32::MAX`"),
        )
        .expect("Audio device should be available");
    }

    /// Allocates exact number of sound channels. Any channels that have id greater than or equal to `channels` will be stopped automatically.
    ///
    /// By default, there are 8 channels that are available.
    ///
    /// # Panics
    /// There can be at most `i32::MAX` sound channels, so `TryFromIntError` will be returned when passing value that exceeds the limit.
    ///
    pub fn allocate_sound_channels(channels: u32) -> Result<(), TryFromIntError> {
        let _ = mixer_allocate_channels(i32::try_from(channels)?);
        Ok(())
    }
}
