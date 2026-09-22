use std::ffi::c_void;
use std::io::Cursor;
use std::mem::size_of;
use std::path::Path;

use ::windows::core::{HSTRING, PCWSTR};
use ::windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ,
};
use ::windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, FILE_FLAGS_AND_ATTRIBUTES,
};
use ::windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use ::windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use base64::Engine;

pub fn product_name(exe_path: &Path) -> Option<String> {
    let block = read_version_block(exe_path)?;
    let (language, code_page) = first_translation(&block)?;
    ["ProductName", "FileDescription"]
        .into_iter()
        .find_map(|key| {
            query_string(
                &block,
                &format!("\\StringFileInfo\\{language:04x}{code_page:04x}\\{key}"),
            )
        })
}

fn read_version_block(exe_path: &Path) -> Option<Vec<u8>> {
    let path = HSTRING::from(exe_path.as_os_str());
    // SAFETY: `path` is a valid null-terminated wide string for the duration of both calls.
    let size = unsafe { GetFileVersionInfoSizeW(&path, None) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: `block` has exactly `size` writable bytes.
    unsafe { GetFileVersionInfoW(&path, None, size, block.as_mut_ptr() as *mut c_void) }.ok()?;
    Some(block)
}

fn first_translation(block: &[u8]) -> Option<(u16, u16)> {
    let (pointer, length) = query_value(block, "\\VarFileInfo\\Translation")?;
    if (length as usize) < 2 * size_of::<u16>() {
        return None;
    }
    // SAFETY: VerQueryValueW guarantees at least `length` bytes at `pointer`, inside `block`.
    let pair = unsafe { std::slice::from_raw_parts(pointer as *const u16, 2) };
    Some((pair[0], pair[1]))
}

fn query_string(block: &[u8], sub_block: &str) -> Option<String> {
    let (pointer, length) = query_value(block, sub_block)?;
    // SAFETY: for string values VerQueryValueW returns `length` UTF-16 units inside `block`.
    let units = unsafe { std::slice::from_raw_parts(pointer as *const u16, length as usize) };
    let value = String::from_utf16_lossy(units);
    let value = value.trim_end_matches('\0').trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn query_value(block: &[u8], sub_block: &str) -> Option<(*const c_void, u32)> {
    let sub_block = HSTRING::from(sub_block);
    let mut pointer: *mut c_void = std::ptr::null_mut();
    let mut length = 0u32;
    // SAFETY: `block` is a version-info buffer filled by GetFileVersionInfoW.
    let found = unsafe {
        VerQueryValueW(
            block.as_ptr() as *const c_void,
            &sub_block,
            &mut pointer,
            &mut length,
        )
    };
    (found.as_bool() && !pointer.is_null() && length > 0)
        .then_some((pointer as *const c_void, length))
}

pub fn icon_png_base64(exe_path: &Path) -> Option<String> {
    let icon = OwnedIcon(load_large_icon(exe_path)?);
    let (width, height, pixels) = icon_to_rgba(icon.0)?;
    let image = image::RgbaImage::from_raw(width, height, pixels)?;

    let mut png = Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(png.into_inner()))
}

fn load_large_icon(exe_path: &Path) -> Option<HICON> {
    let path = HSTRING::from(exe_path.as_os_str());
    let mut info = SHFILEINFOW::default();
    // SAFETY: `info` is a valid SHFILEINFOW whose size is passed; `path` outlives the call.
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    (result != 0 && !info.hIcon.is_invalid()).then_some(info.hIcon)
}

fn icon_to_rgba(icon: HICON) -> Option<(u32, u32, Vec<u8>)> {
    let mut icon_info = ICONINFO::default();
    // SAFETY: `icon` is a live icon handle; `icon_info` is a valid out-pointer.
    unsafe { GetIconInfo(icon, &mut icon_info) }.ok()?;
    let color = OwnedBitmap(icon_info.hbmColor);
    let _mask = OwnedBitmap(icon_info.hbmMask);
    if color.0.is_invalid() {
        return None;
    }

    let mut bitmap = BITMAP::default();
    // SAFETY: `bitmap` is a BITMAP-sized buffer for a live bitmap handle.
    let copied = unsafe {
        GetObjectW(
            HGDIOBJ(color.0 .0),
            size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut c_void),
        )
    };
    if copied == 0 {
        return None;
    }
    let (width, height) = (bitmap.bmWidth as u32, bitmap.bmHeight as u32);

    let mut header = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // negative height = top-down rows
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    // SAFETY: the screen DC is released right after; `pixels` holds width*height 32-bit pixels.
    let lines = unsafe {
        let screen = GetDC(None);
        let lines = GetDIBits(
            screen,
            color.0,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut header,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, screen);
        lines
    };
    if lines == 0 {
        return None;
    }

    bgra_to_rgba(&mut pixels);
    Some((width, height, pixels))
}

fn bgra_to_rgba(pixels: &mut [u8]) {
    // Legacy icons carry no alpha channel; treat them as fully opaque instead of invisible.
    let has_alpha = pixels.chunks_exact(4).any(|pixel| pixel[3] != 0);
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
        if !has_alpha {
            pixel[3] = u8::MAX;
        }
    }
}

struct OwnedIcon(HICON);

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        // SAFETY: the icon was created by SHGetFileInfoW and is destroyed exactly once.
        let _ = unsafe { DestroyIcon(self.0) };
    }
}

struct OwnedBitmap(HBITMAP);

impl Drop for OwnedBitmap {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: GetIconInfo hands ownership of its bitmaps to the caller.
            let _ = unsafe { DeleteObject(HGDIOBJ(self.0 .0)) };
        }
    }
}
