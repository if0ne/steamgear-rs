use crate::core::{callback::CallbackTyped, structs::AppId};

use steamgear_sys as sys;

#[derive(Clone, Debug)]
pub struct DlcInformation {
    pub dlc_id: AppId,
    pub dlc_name: String,
    pub available: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct DlcDownloadProgress {
    pub downloaded: u64,
    pub total: u64,
}

#[derive(Clone, Debug)]
pub struct FileDetails {
    pub file_size: u64,
    pub sha1: [u8; 20],
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub struct FileNotFound;

impl std::fmt::Display for FileNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "File was not found")
    }
}

impl CallbackTyped for FileDetails {
    const TYPE: u32 = sys::FileDetailsResult_t_k_iCallback as u32;

    type Raw = sys::FileDetailsResult_t;
    type Mapped = Result<Self, FileNotFound>;

    fn from_raw(raw: Self::Raw) -> Self::Mapped {
        if raw.m_eResult == sys::EResult_k_EResultOK {
            Ok(Self {
                file_size: raw.m_ulFileSize,
                sha1: raw.m_FileSHA,
            })
        } else {
            Err(FileNotFound)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TrialTime {
    pub allowed: u32,
    pub played: u32,
}
