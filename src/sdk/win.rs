use std::error::Error;
use winapi::shared::minwindef::{BOOL, DWORD, FALSE, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ};
use winapi::um::synchapi::OpenEventA;
use winapi::um::winbase::OpenFileMappingA;
use winapi::um::winnt::{HANDLE, PAGE_READONLY, SYNCHRONIZE};

pub const DATA_READY_EVENT: &str = "Local\\IRSDKDataValidEvent";
pub const MEMORY_MAPPED_FILE: &str = "Local\\IRSDKMemMapFileName";

pub struct Sdk {
    memory_mapped_file: HANDLE,
    shared_memory: LPVOID,
    data_ready_event: HANDLE,
}

impl Sdk {
    pub fn new() -> Result<Self, dyn Error> {
        unsafe {
            let memory_mapped_file =
                OpenFileMappingA(PAGE_READONLY, FALSE, MEMORY_MAPPED_FILE as *const i8);
            if memory_mapped_file.is_null() {
                return Err("Failed to open memory mapped file.");
            }

            let shared_memory = MapViewOfFile(memory_mapped_file, FILE_MAP_READ, 0, 0, 0);
            if shared_memory.is_null() {
                return Err("Failed to create view of memory mapped file.");
            }

            let data_ready_event = OpenEventA(SYNCHRONIZE, FALSE, DATA_READY_EVENT as *const i8);
            if data_ready_event.is_null() {
                return Err("Failed to subscribe to data ready event.");
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
