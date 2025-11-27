use libloading::{Library, Symbol};
use log::{debug, error, info};
use std::ffi::{CStr, CString};
use std::ptr;
use std::sync::Arc;

// Type definitions for pcap functions
type PcapFindAllDevs = unsafe extern "C" fn(*mut *mut PcapIf, *mut i8) -> i32;
type PcapFreeAllDevs = unsafe extern "C" fn(*mut PcapIf);
type PcapOpenLive = unsafe extern "C" fn(*const i8, i32, i32, i32, *mut i8) -> *mut PcapT;
type PcapClose = unsafe extern "C" fn(*mut PcapT);
type PcapNextEx = unsafe extern "C" fn(*mut PcapT, *mut *mut PcapPkthdr, *mut *const u8) -> i32;
type PcapGetErr = unsafe extern "C" fn(*mut PcapT) -> *mut i8;
type PcapDataLink = unsafe extern "C" fn(*mut PcapT) -> i32;

#[repr(C)]
pub struct PcapIf {
    pub next: *mut PcapIf,
    pub name: *mut i8,
    pub description: *mut i8,
    pub addresses: *mut PcapAddr,
    pub flags: u32,
}

#[repr(C)]
pub struct PcapAddr {
    pub next: *mut PcapAddr,
    pub addr: *mut libc::sockaddr,
    pub netmask: *mut libc::sockaddr,
    pub broadaddr: *mut libc::sockaddr,
    pub dstaddr: *mut libc::sockaddr,
}

#[repr(C)]
pub struct PcapPkthdr {
    pub ts: libc::timeval,
    pub caplen: u32,
    pub len: u32,
}

pub enum PcapT {}

pub struct NpcapContext {
    lib: Arc<Library>,
}

impl NpcapContext {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let lib = Library::new("wpcap.dll")
                .map_err(|e| format!("Failed to load wpcap.dll: {}", e))?;
            Ok(Self { lib: Arc::new(lib) })
        }
    }

    pub fn list_devices(&self) -> Result<Vec<Device>, String> {
        let mut devices = Vec::new();
        unsafe {
            let find_all_devs: Symbol<PcapFindAllDevs> = self
                .lib
                .get(b"pcap_findalldevs")
                .map_err(|e| e.to_string())?;
            let free_all_devs: Symbol<PcapFreeAllDevs> = self
                .lib
                .get(b"pcap_freealldevs")
                .map_err(|e| e.to_string())?;

            let mut alldevs: *mut PcapIf = ptr::null_mut();
            let mut errbuf = [0i8; 256];

            if find_all_devs(&mut alldevs, errbuf.as_mut_ptr()) == -1 {
                return Err(CStr::from_ptr(errbuf.as_ptr())
                    .to_string_lossy()
                    .into_owned());
            }

            let mut curr = alldevs;
            while !curr.is_null() {
                let name = CStr::from_ptr((*curr).name).to_string_lossy().into_owned();
                let description = if !(*curr).description.is_null() {
                    Some(
                        CStr::from_ptr((*curr).description)
                            .to_string_lossy()
                            .into_owned(),
                    )
                } else {
                    None
                };

                devices.push(Device { name, description });
                curr = (*curr).next;
            }

            free_all_devs(alldevs);
        }
        Ok(devices)
    }
}

#[derive(Debug, Clone, serde::Serialize, specta::Type)]
pub struct Device {
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn get_network_devices() -> Result<Vec<Device>, String> {
    let context = NpcapContext::new()?;
    context.list_devices()
}

#[tauri::command]
#[specta::specta]
pub fn check_npcap_status() -> bool {
    NpcapContext::new().is_ok()
}

pub struct NpcapCapture {
    handle: *mut PcapT,
    lib: Arc<Library>,
}

unsafe impl Send for NpcapCapture {}

impl NpcapCapture {
    pub fn new(device_name: &str) -> Result<Self, String> {
        let context = NpcapContext::new()?;
        unsafe {
            let open_live: Symbol<PcapOpenLive> = context
                .lib
                .get(b"pcap_open_live")
                .map_err(|e| e.to_string())?;
            let get_err: Symbol<PcapGetErr> =
                context.lib.get(b"pcap_geterr").map_err(|e| e.to_string())?;

            let device_c = CString::new(device_name).map_err(|e| e.to_string())?;
            let mut errbuf = [0i8; 256];

            // Snaplen 65536, promiscuous 1, timeout 1000ms
            let handle = open_live(device_c.as_ptr(), 65536, 1, 1000, errbuf.as_mut_ptr());

            if handle.is_null() {
                return Err(CStr::from_ptr(errbuf.as_ptr())
                    .to_string_lossy()
                    .into_owned());
            }

            Ok(Self {
                handle,
                lib: context.lib,
            })
        }
    }

    pub fn next_packet(&self) -> Result<Option<Vec<u8>>, String> {
        unsafe {
            let next_ex: Symbol<PcapNextEx> =
                self.lib.get(b"pcap_next_ex").map_err(|e| e.to_string())?;

            let mut header: *mut PcapPkthdr = ptr::null_mut();
            let mut data: *const u8 = ptr::null();

            let res = next_ex(self.handle, &mut header, &mut data);

            match res {
                1 => {
                    // Success
                    let len = (*header).caplen as usize;
                    let mut packet_data = Vec::with_capacity(len);
                    ptr::copy_nonoverlapping(data, packet_data.as_mut_ptr(), len);
                    packet_data.set_len(len);
                    Ok(Some(packet_data))
                }
                0 => Ok(None), // Timeout
                -1 => Err("Error reading packet".to_string()),
                -2 => Ok(None), // EOF
                _ => Err(format!("Unknown pcap_next_ex return code: {}", res)),
            }
        }
    }
}

impl Drop for NpcapCapture {
    fn drop(&mut self) {
        unsafe {
            if let Ok(close) = self.lib.get::<PcapClose>(b"pcap_close") {
                close(self.handle);
            }
        }
    }
}
