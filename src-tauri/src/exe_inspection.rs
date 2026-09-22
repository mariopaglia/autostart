use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::models::ExeInfo;
use crate::platform;

pub fn inspect_exe(exe_path: &Path) -> AppResult<ExeInfo> {
    if !exe_path.is_file() {
        return Err(AppError::ExecutableNotFound(exe_path.display().to_string()));
    }
    if cfg!(windows) && !has_exe_extension(exe_path) {
        return Err(AppError::Validation(format!(
            "{} is not an .exe file",
            exe_path.display()
        )));
    }

    let process_name = file_name(exe_path);
    let product_name = platform::product_name(exe_path).unwrap_or_else(|| file_stem(exe_path));

    Ok(ExeInfo {
        process_name,
        product_name,
        icon_base64: platform::icon_png_base64(exe_path),
    })
}

fn has_exe_extension(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_reported() {
        let error = inspect_exe(Path::new("/definitely/missing/app.exe")).unwrap_err();
        assert!(matches!(error, AppError::ExecutableNotFound(_)));
    }

    #[test]
    fn name_falls_back_to_file_name_without_metadata() {
        let dir = tempfile::tempdir().expect("temp dir");
        let exe = dir.path().join("Navigraph Charts.exe");
        std::fs::write(&exe, b"not a real binary").expect("write");

        let info = inspect_exe(&exe).expect("inspect");

        assert_eq!(info.process_name, "Navigraph Charts.exe");
        assert_eq!(info.product_name, "Navigraph Charts");
    }
}
