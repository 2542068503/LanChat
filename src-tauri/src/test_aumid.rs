use windows::core::{HSTRING, PCWSTR};
use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;
pub fn test() {
    let aumid = HSTRING::from("com.test");
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(&aumid);
    }
}
