#[cfg(target_os = "windows")]
pub unsafe fn set_window_aumid(hwnd: isize, aumid: &str) {
    use std::ffi::c_void;
    use windows::core::{GUID, PWSTR};
    use windows::Win32::Foundation::{HWND, PROPERTYKEY};
    use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0};
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, CoTaskMemAlloc};
    use windows::Win32::System::Variant::VT_LPWSTR;
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};

    let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    let needs_uninit = hr.is_ok();
    
    let hwnd = HWND(hwnd as *mut c_void);
    
    if let Ok(store) = SHGetPropertyStoreForWindow::<IPropertyStore>(hwnd) {
        let pkey = PROPERTYKEY {
            fmtid: GUID::from_u128(0x9F4C2855_9F79_4B39_A8D0_E1D42DE1D5F3),
            pid: 5,
        };
        
        let mut wide: Vec<u16> = aumid.encode_utf16().collect();
        wide.push(0);
        
        let cb = wide.len() * std::mem::size_of::<u16>();
        let ptr = CoTaskMemAlloc(cb) as *mut u16;
        if !ptr.is_null() {
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            
            let propvar = PROPVARIANT {
                Anonymous: PROPVARIANT_0 {
                    Anonymous: core::mem::ManuallyDrop::new(PROPVARIANT_0_0 {
                        vt: VT_LPWSTR,
                        wReserved1: 0,
                        wReserved2: 0,
                        wReserved3: 0,
                        Anonymous: PROPVARIANT_0_0_0 {
                            pwszVal: PWSTR(ptr),
                        },
                    }),
                },
            };
            
            let _ = store.SetValue(&pkey, &propvar);
            let _ = store.Commit();
        }
    }
    
    if needs_uninit {
        CoUninitialize();
    }
}

