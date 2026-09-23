use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use ::windows::core::{Interface, GUID, HSTRING};
use ::windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use ::windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, STGM_READ,
};
use ::windows::Win32::UI::Shell::{
    FOLDERID_CommonPrograms, FOLDERID_Desktop, FOLDERID_Programs, FOLDERID_PublicDesktop,
    IShellLinkW, SHGetKnownFolderPath, ShellLink, KF_FLAG_DEFAULT,
};

use crate::platform::ShortcutTarget;

const TEXT_BUFFER_LEN: usize = 1024;

/// Start Menu folders come first so their shortcut names win over desktop duplicates.
const SHORTCUT_FOLDERS: [GUID; 4] = [
    FOLDERID_CommonPrograms,
    FOLDERID_Programs,
    FOLDERID_PublicDesktop,
    FOLDERID_Desktop,
];

pub fn shortcut_folders() -> Vec<PathBuf> {
    SHORTCUT_FOLDERS.iter().filter_map(known_folder).collect()
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
    // SAFETY: `id` points to a valid GUID; the returned buffer is freed right after copying it.
    unsafe {
        let buffer = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let path = PathBuf::from(OsString::from_wide(buffer.as_wide()));
        CoTaskMemFree(Some(buffer.0 as *const _));
        Some(path)
    }
}

/// Holds one COM apartment and one ShellLink object so a whole Start Menu scan reuses them.
pub struct ShortcutResolver {
    link: IShellLinkW,
    file: IPersistFile,
    // Declared last: fields drop in order, so the COM objects are released before CoUninitialize.
    _apartment: ComApartment,
}

impl ShortcutResolver {
    pub fn new() -> Option<Self> {
        let apartment = ComApartment::enter()?;
        // SAFETY: COM is initialized on this thread for as long as the resolver lives.
        let link: IShellLinkW =
            unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.ok()?;
        let file = link.cast::<IPersistFile>().ok()?;
        Some(Self {
            link,
            file,
            _apartment: apartment,
        })
    }

    /// Reads the stored target without `IShellLinkW::Resolve`, which may show UI or search the disk.
    pub fn resolve(&self, shortcut: &Path) -> Option<ShortcutTarget> {
        let path = HSTRING::from(shortcut.as_os_str());
        // SAFETY: `path` is a valid wide string for the call; the object belongs to this thread.
        unsafe { self.file.Load(&path, STGM_READ) }.ok()?;

        let mut buffer = [0u16; TEXT_BUFFER_LEN];
        // SAFETY: GetPath writes at most `buffer.len()` UTF-16 units; no find data is requested.
        unsafe {
            self.link
                .GetPath(&mut buffer, std::ptr::null_mut(), 0)
                .ok()?
        };
        let target_path = PathBuf::from(wide_to_string(&buffer)?);

        // SAFETY: GetArguments writes at most `buffer.len()` UTF-16 units.
        let args = unsafe { self.link.GetArguments(&mut buffer) }
            .ok()
            .and_then(|()| wide_to_string(&buffer));
        // SAFETY: GetWorkingDirectory writes at most `buffer.len()` UTF-16 units.
        let working_dir = unsafe { self.link.GetWorkingDirectory(&mut buffer) }
            .ok()
            .and_then(|()| wide_to_string(&buffer))
            .map(PathBuf::from);

        Some(ShortcutTarget {
            target_path,
            args,
            working_dir,
        })
    }
}

fn wide_to_string(buffer: &[u16]) -> Option<String> {
    let length = buffer
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(buffer.len());
    let text = String::from_utf16_lossy(&buffer[..length]);
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_owned())
}

struct ComApartment {
    owns_initialization: bool,
}

impl ComApartment {
    fn enter() -> Option<Self> {
        // SAFETY: no reserved pointer is passed; a successful call is balanced in Drop.
        let result =
            unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        // The thread already joined a multithreaded apartment: COM is usable but not ours to close.
        if result == RPC_E_CHANGED_MODE {
            return Some(Self {
                owns_initialization: false,
            });
        }
        result.ok().ok()?;
        Some(Self {
            owns_initialization: true,
        })
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.owns_initialization {
            // SAFETY: balances the successful CoInitializeEx made by `enter` on this same thread.
            unsafe { CoUninitialize() };
        }
    }
}
