use std::ffi::CString;
use std::io::{Error, ErrorKind};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::shared::winerror::ERROR_FILE_NOT_FOUND;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ};
use winapi::um::synchapi::OpenEventA;
use winapi::um::winbase::OpenFileMappingA;
use winapi::um::winnt::{HANDLE, PAGE_READONLY, SYNCHRONIZE};

pub struct Sdk {
    memory_mapped_file: HANDLE,
    shared_memory: LPVOID,
    data_ready_event: HANDLE,
}

impl Sdk {
    pub fn new() -> Result<Self, Error> {
        let memory_mapped_file = match open_file_mapping(&memory_mapped_file_name()) {
            Ok(handle) => handle,
            Err(error) => return Err(error),
        };

        let shared_memory: LPVOID;
        let data_ready_event: HANDLE;

        unsafe {
            shared_memory = MapViewOfFile(memory_mapped_file, FILE_MAP_READ, 0, 0, 0);
            if shared_memory.is_null() {
                return Err(Error::new(
                    ErrorKind::NotConnected,
                    "Failed to create view of memory mapped file.",
                ));
            }

            data_ready_event = OpenEventA(SYNCHRONIZE, FALSE, data_ready_event_name().as_ptr());
            if data_ready_event.is_null() {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to subscribe to data ready event.",
                ));
            }
        }

        Ok(Self {
            memory_mapped_file,
            shared_memory,
            data_ready_event,
        })
    }
}

impl Drop for Sdk {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.data_ready_event);
            UnmapViewOfFile(self.shared_memory);
            CloseHandle(self.memory_mapped_file);
        }
    }
}

fn memory_mapped_file_name() -> CString {
    CString::new("Local\\IRSDKMemMapFileName")
        .expect("Failed to create CString from the hardcoded memory mapped file name.")
}

fn data_ready_event_name() -> CString {
    CString::new("Local\\IRSDKDataValidEvent")
        .expect("Failed to create a CString from the hardcoded event name.")
}

fn open_file_mapping(name: &CString) -> Result<HANDLE, Error> {
    let memory_mapped_file;

    unsafe {
        memory_mapped_file = OpenFileMappingA(PAGE_READONLY, FALSE, name.as_ptr());

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

#[cfg(test)]
mod tests {
    use super::open_file_mapping;
    use std::ffi::CString;
    use std::io::{Error, ErrorKind};
    use winapi::shared::ntdef::NULL;
    use winapi::um::errhandlingapi::GetLastError;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
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
}
