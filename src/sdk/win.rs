use crate::sdk::IRacingClient;
use std::ffi::CString;
use std::io::{Error, ErrorKind};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::shared::winerror::{ERROR_ACCESS_DENIED, ERROR_FILE_NOT_FOUND};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ};
use winapi::um::synchapi::OpenEventA;
use winapi::um::winbase::OpenFileMappingA;
use winapi::um::winnt::{HANDLE, SYNCHRONIZE};

pub const DATA_READY_EVENT: &str = "Local\\IRSDKDataValidEvent";
pub const MEMORY_MAPPED_FILE: &str = "Local\\IRSDKMemMapFileName";

pub struct WinClient {
    memory_mapped_file: HANDLE,
    shared_memory: LPVOID,
    data_ready_event: HANDLE,
}

impl IRacingClient for WinClient {
    fn new() -> Result<Self, Error> {
        let memory_mapped_file = match open_file_mapping(&memory_mapped_file_name()) {
            Ok(handle) => handle,
            Err(error) => return Err(error),
        };

        let shared_memory = match map_view_of_file(memory_mapped_file) {
            Ok(pointer) => pointer,
            Err(error) => return Err(error),
        };

        let data_ready_event = match open_event(&data_ready_event_name()) {
            Ok(data_ready_event) => data_ready_event,
            Err(error) => return Err(error),
        };

        Ok(WinClient {
            memory_mapped_file,
            shared_memory,
            data_ready_event,
        })
    }

    fn run(&self) {
        // Wait for data ready event from Windows
        // Check if data in memory has been updated
        // Copy data to new location to prevent it from being changed
        // Decode copied memory
        // Push decoded update through channel
    }
}

impl Drop for WinClient {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.data_ready_event);
            UnmapViewOfFile(self.shared_memory);
            CloseHandle(self.memory_mapped_file);
        }
    }
}

fn memory_mapped_file_name() -> CString {
    CString::new(MEMORY_MAPPED_FILE)
        .expect("Failed to create CString from the hardcoded memory mapped file name.")
}

fn data_ready_event_name() -> CString {
    CString::new(DATA_READY_EVENT)
        .expect("Failed to create a CString from the hardcoded event name.")
}

fn open_file_mapping(name: &CString) -> Result<HANDLE, Error> {
    let memory_mapped_file;

    unsafe {
        memory_mapped_file = OpenFileMappingA(FILE_MAP_READ, FALSE, name.as_ptr());

        if memory_mapped_file.is_null() {
            let error = GetLastError();

            match error {
                ERROR_FILE_NOT_FOUND => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        "Failed to open memory mapped file.",
                    ))
                }
                error => panic!(
                    "Unexpected error occured while opening a file mapping. Error code: {}.",
                    error
                ),
            }
        }
    }

    Ok(memory_mapped_file)
}

fn map_view_of_file(memory_mapped_file: HANDLE) -> Result<LPVOID, Error> {
    let shared_memory;

    unsafe {
        shared_memory = MapViewOfFile(memory_mapped_file, FILE_MAP_READ, 0, 0, 0);

        if shared_memory.is_null() {
            let error = GetLastError();

            match error {
                ERROR_ACCESS_DENIED => {
                    return Err(Error::new(
                        ErrorKind::PermissionDenied,
                        "Failed to created view of mapped memory file due to denied access.",
                    ))
                }
                error => panic!(
                    "Unexpected error occured while creating a view of a memory mapped file. Error code: {}.",
                    error
                ),
            }
        }
    }

    Ok(shared_memory)
}

fn open_event(name: &CString) -> Result<HANDLE, Error> {
    let data_ready_event: HANDLE;

    unsafe {
        data_ready_event = OpenEventA(SYNCHRONIZE, FALSE, name.as_ptr());

        if data_ready_event.is_null() {
            let error = GetLastError();

            match error {
                ERROR_FILE_NOT_FOUND => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        "Failed to open data ready event.",
                    ))
                }
                error => panic!(
                    "Unexpected error occured while opening a file mapping. Error code: {}.",
                    error
                ),
            }
        }
    }

    Ok(data_ready_event)
}

#[cfg(test)]
mod tests {
    use super::{map_view_of_file, open_event, open_file_mapping};
    use std::ffi::CString;
    use std::io::{Error, ErrorKind};
    use winapi::shared::minwindef::FALSE;
    use winapi::shared::ntdef::NULL;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::memoryapi::UnmapViewOfFile;
    use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
    use winapi::um::synchapi::CreateEventA;
    use winapi::um::winbase::CreateFileMappingA;
    use winapi::um::winnt::HANDLE;
    use winapi::um::winnt::PAGE_READWRITE;

    fn create_file_mapping(name: &CString) -> Result<HANDLE, Error> {
        let file_mapping;

        unsafe {
            file_mapping = CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                NULL as *mut SECURITY_ATTRIBUTES,
                PAGE_READWRITE,
                0,
                1024,
                name.as_ptr(),
            );

            if file_mapping.is_null() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to create file mapping. Error {}.", GetLastError()),
                ));
            }
        }

        Ok(file_mapping)
    }

    #[test]
    fn open_existing_file_mapping() {
        let name = CString::new("open_existing_file_mapping").unwrap();

        let existing_file_mapping = create_file_mapping(&name).unwrap();
        let file_mapping = open_file_mapping(&name).unwrap();

        unsafe {
            CloseHandle(file_mapping);
            CloseHandle(existing_file_mapping);
        }
    }

    #[test]
    fn open_missing_file_mapping() {
        let name = CString::new("open_missing_file_mapping").unwrap();

        match open_file_mapping(&name) {
            Ok(_) => panic!("Test should fail due to missing file mapping"),
            Err(error) => assert_eq!(ErrorKind::NotFound, error.kind()),
        }
    }

    #[test]
    fn map_view_of_existing_file() {
        let name = CString::new("map_view_of_existing_file").unwrap();

        let file = create_file_mapping(&name).unwrap();
        let mapping = open_file_mapping(&name).unwrap();
        let view = map_view_of_file(mapping).unwrap();

        unsafe {
            UnmapViewOfFile(view);
            CloseHandle(mapping);
            CloseHandle(file);
        }
    }

    #[test]
    fn open_existing_event() {
        let name = CString::new("open_existing_event").unwrap();

        let existing_event;

        unsafe {
            existing_event = CreateEventA(
                NULL as *mut SECURITY_ATTRIBUTES,
                FALSE,
                FALSE,
                name.as_ptr(),
            );

            if existing_event.is_null() {
                panic!("Failed to create event. Error {}.", GetLastError());
            }
        }

        let event = open_event(&name).unwrap();

        unsafe {
            CloseHandle(event);
            CloseHandle(existing_event);
        }
    }

    #[test]
    fn open_missing_event() {
        let name = CString::new("open_missing_event").unwrap();

        match open_event(&name) {
            Ok(_) => panic!("Test should fail due to missing event"),
            Err(error) => assert_eq!(ErrorKind::NotFound, error.kind()),
        }
    }
}
