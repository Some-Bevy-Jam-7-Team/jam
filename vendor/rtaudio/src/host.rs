use crate::error::{RtAudioError, RtAudioErrorType};
use crate::wrapper::RtAudioWrapper;
use crate::{Api, DeviceID, DeviceInfo, SessionID, StreamConfig, StreamHandle};

#[cfg(all(feature = "log", not(feature = "tracing")))]
use log::warn;
#[cfg(feature = "tracing")]
use tracing::warn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindDeviceInfo {
    /// The index this device appears in [`Host::devices()`].
    pub index: usize,
    /// The current session ID for this device (this may differ across reboots).
    pub session_id: SessionID,
}

/// An RtAudio Host instance. This is used to enumerate audio devices before
/// opening a stream.
pub struct Host {
    pub(crate) wrapper: RtAudioWrapper,
    pub(crate) devices: Vec<DeviceInfo>,
}

impl Host {
    /// Create a new RtAudio Host with the given API. This host is used to
    /// enumerate audio devices before opening a stream.
    ///
    /// If `Api::Unspecified` is used, then the best one for the system will
    /// automatically be chosen.
    pub fn new(api: Api) -> Result<Self, RtAudioError> {
        let mut new_self = Self {
            wrapper: RtAudioWrapper::new(api)?,
            devices: Vec::new(),
        };

        new_self.refresh_devices();

        Ok(new_self)
    }

    /// Whether or not to print extra warnings to the terminal output.
    ///
    /// By default this is set to `false`.
    pub fn show_warnings(&mut self, show: bool) {
        self.wrapper.show_warnings(show);
    }

    /// The API being used by this instance.
    pub fn api(&self) -> Api {
        self.wrapper.api()
    }

    /// Refresh the list of audio devices.
    ///
    /// This will invalidate any device indexes.
    pub fn refresh_devices(&mut self) {
        let num_devices = self.wrapper.num_devices();
        self.devices.clear();

        for i in 0..num_devices {
            // Safe because `self.raw` is gauranteed to not be null.
            let session_id = self.wrapper.get_device_session_id_at_index(i);

            let info = if session_id.is_none() {
                Err(RtAudioError {
                    type_: RtAudioErrorType::InvalidParamter,
                    msg: Some(format!("Could not find device at index {}", i)),
                })
            } else if let Err(e) = self.wrapper.check_for_error() {
                Err(e)
            } else {
                self.wrapper.get_device_info(session_id.unwrap())
            };

            match info {
                Ok(info) => self.devices.push(info),
                Err(e) => {
                    #[cfg(not(any(feature = "tracing", feature = "log")))]
                    let _ = e;

                    #[cfg(any(feature = "tracing", feature = "log"))]
                    warn!("Error while scanning audio device at index {}: {}", i, e);
                }
            }
        }
    }

    /// Find the index and session ID for the device with the given ID.
    ///
    /// Returns `None` if the device was not found.
    pub fn find_device(&self, id: &DeviceID) -> Option<FindDeviceInfo> {
        let mut first_matching_name_idx = None;
        for (i, device_info) in self.devices.iter().enumerate() {
            let name_matches = device_info.name() == &id.name;

            if first_matching_name_idx.is_none() && name_matches {
                first_matching_name_idx = Some(i);
            }

            if name_matches && device_info.id.session_id == id.session_id {
                return Some(FindDeviceInfo {
                    index: i,
                    session_id: id.session_id,
                });
            }
        }

        first_matching_name_idx.map(|i| FindDeviceInfo {
            index: i,
            session_id: self.devices[i].id.session_id,
        })
    }

    /// Get the list of available audio devices.
    pub fn devices(&self) -> &[DeviceInfo] {
        &self.devices
    }

    /// Retrieve an iterator over the available output audio devices.
    pub fn iter_output_devices<'a>(&'a self) -> impl Iterator<Item = &'a DeviceInfo> {
        self.devices.iter().filter(|d| d.output_channels > 0)
    }

    /// Retrieve an iterator over the available input audio devices.
    pub fn iter_input_devices<'a>(&'a self) -> impl Iterator<Item = &'a DeviceInfo> {
        self.devices.iter().filter(|d| d.input_channels > 0)
    }

    /// Retrieve an iterator over the available duplex audio devices.
    pub fn iter_duplex_devices<'a>(&'a self) -> impl Iterator<Item = &'a DeviceInfo> {
        self.devices.iter().filter(|d| d.duplex_channels > 0)
    }

    /// Get the index of the default input device.
    ///
    /// Return `None` if no default input device was found.
    pub fn default_input_device_index(&self) -> Option<usize> {
        self.devices
            .iter()
            .position(|d| d.is_default_input && d.input_channels > 0)
    }

    /// Get the index of the default output device.
    ///
    /// Return `None` if no default output device was found.
    pub fn default_output_device_index(&self) -> Option<usize> {
        self.devices
            .iter()
            .position(|d| d.is_default_output && d.output_channels > 0)
    }

    /// Get the index of the default duplex device.
    ///
    /// Return `None` if no default duplex device was found.
    pub fn default_duplex_device_index(&self) -> Option<usize> {
        self.devices
            .iter()
            .position(|d| (d.is_default_input || d.is_default_output) && d.duplex_channels > 0)
    }

    /// Open a new audio stream.
    ///
    /// Multiple streams can be opened at the same time using multiple [`Host`]s.
    pub fn open_stream(self, config: &StreamConfig) -> Result<StreamHandle, (Self, RtAudioError)> {
        StreamHandle::new(self, config)
    }
}

impl Default for Host {
    fn default() -> Self {
        Self::new(Api::Unspecified).unwrap()
    }
}

impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host").finish()
    }
}
