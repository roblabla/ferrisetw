fn main() {
    windows_macros::build!(
        Windows::Win32::System::Diagnostics::Etw::*,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::System::SystemServices::{
            PSTR, MAX_PATH, VER_GREATER_EQUAL
        },
        Windows::Win32::System::Memory::LocalFree,
        Windows::Win32::System::OleAutomation::{
            SysStringLen, BSTR
        },
        Windows::Win32::System::WindowsProgramming::{
            FILETIME, GetSystemTimeAsFileTime, OSVERSIONINFOEXA,
            VerifyVersionInfoA, VerSetConditionMask, VER_FLAGS,
            // We need all WindowsProgramming for VER_FLAGS variants...
            VER_MAJORVERSION, VER_MINORVERSION, VER_SERVICEPACKMAJOR
        },
        Windows::Win32::Security::{ConvertSidToStringSidA, PSID},
    );
}
