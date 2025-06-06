//Even though this code is in Rust, it is not exactly the Rust that queryies temperature sensors
//of cpu and gpu. Rust is used to process, filter and format the data that are arriving
//from Objective C functions that are responsible for fetching the raw sensor data
//As a consequence, the core functionality lives in the `unsafe` block, where  Objective C methods are wrapped in Rust
//it is unsafe not because there is something nasty there, but because Rust does not have any means
// to verify safety for functions controlled by another language

//imports from Rust std
use std::{collections::HashMap, mem::size_of, os::raw::c_void};

//external Rust library that is meant to communicate with MacOS Core Foundation
//This crate provides wrappers around the underlying CoreFoundation types and functions that are available on Apple’s operating systems.
//https://docs.rs/core-foundation/latest/core_foundation/
use core_foundation::dictionary::{CFDictionaryRef, CFMutableDictionaryRef};

pub type WithError<T> = Result<T, Box<dyn std::error::Error>>;

// That is IOKit: https://developer.apple.com/documentation/iokit
// That is what is needed to communicate and do something with hw devices and is meant for apple developers
//cpu, gpu, sensors are exatcly hardware

// ########### SECTION IOKit Bindings ###############

// The following line tells the Rust compiler (his linker) to link this crate against Apple’s (macOS) IOKit.framework,
//(to pull in the Objective C library IOKit.framework )
//what is it?
// $ otool -L /usr/bin/powermetrics
// output:
// /usr/bin/powermetrics:
//...
// /System/Library/Frameworks/CoreFoundation.framework/Versions/A/CoreFoundation (compatibility version 150.0.0, current version 3502.0.0)
// ..
// /System/Library/Frameworks/IOKit.framework/Versions/A/IOKit (compatibility version 1.0.0, current version 275.0.0)
#[link(name = "IOKit", kind = "framework")]
#[rustfmt::skip]

//I added some explanation taken from Apple Developers documentation on IOKit to demonstrate that 
//all functions that are used are of read only nature, anyway if there was something to write 
//all of this would have not run withou elevated sudo permissions
unsafe extern "C" {
   //create a matching dictionary that specifies an IOService class match (CFMutableDictionaryRef)
   fn IOServiceMatching(name: *const i8) -> CFMutableDictionaryRef;
   //looks up a registered IOService object that matches a matching dictionary
   fn IOServiceGetMatchingServices(mainPort: u32, matching: CFDictionaryRef, existing: *mut u32) -> i32;
   //returns the next object in an iteration
   fn IOIteratorNext(iterator: u32) -> u32;
   //returns a C-string name assigned to a registry entry
   fn IORegistryEntryGetName(entry: u32, name: *mut i8) -> i32;
   //releases an object handle previously returned by IOKitLib
   fn IOObjectRelease(obj: u32) -> u32;
}

//iterator is for iterating over existing exposed services to choose the one of interest
pub struct IOServiceIterator {
    existing: u32,
}

impl IOServiceIterator {
    //creating a new iterator for the service name of interest - AppleSMC"
    pub fn new(service_name: &str) -> WithError<Self> {
        let service_name = std::ffi::CString::new(service_name).unwrap();
        let existing = unsafe {
            let service = IOServiceMatching(service_name.as_ptr() as _);
            let mut existing = 0;
            // calling IOKit to get a matching iterator
            if IOServiceGetMatchingServices(0, service, &mut existing) != 0 {
                return Err(format!("{} not found", service_name.to_string_lossy()).into());
            }
            existing
        };

        Ok(Self { existing })
    }
}

impl Drop for IOServiceIterator {
    // if/when the iterator goes out of scope, releasing the IOKit object
    fn drop(&mut self) {
        unsafe {
            IOObjectRelease(self.existing);
        }
    }
}
//each service contains diferent objects,
//$ ioreg is a good place to start to explore
//in this current crate the focus is on:
// $ ioreg -c AppleSMCKeysEndpoint -c SMCEndpoint1 -c AppleSMCKeysEndpoint -l
// this iterator is needed to get it
impl Iterator for IOServiceIterator {
    type Item = (u32, String);

    fn next(&mut self) -> Option<Self::Item> {
        // calling IOIteratorNext to get the next registry entry...
        let next = unsafe { IOIteratorNext(self.existing) };
        // if it returns 0, there are no more services to iterate over
        if next == 0 {
            return None;
        }
        // preparing a buffer to receive the C-string registry name of the object of selected service
        // 128 bytes is defined in Apple’s documentation for registry names
        let mut name = [0; 128];
        // IORegistryEntryGetName returns 0 in ase of success
        if unsafe { IORegistryEntryGetName(next, name.as_mut_ptr()) } != 0 {
            return None;
        }

        let name = unsafe { std::ffi::CStr::from_ptr(name.as_ptr()) };
        let name = name.to_string_lossy().to_string();
        //returns a tuple of (IOService object handle, registry name)
        Some((next, name))
    }
}

// ########### IOKit SMC Bindings section #############
//again, telling Rust linker to link this crate against Apple’s (macOS) IOKit.framework
#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    //The mach task that will request the connection
    fn mach_task_self() -> u32;
    // a request to create a connection to an IOService
    fn IOServiceOpen(device: u32, a: u32, b: u32, c: *mut u32) -> i32;
    fn IOServiceClose(conn: u32) -> i32;
    fn IOConnectCallStructMethod(
        conn: u32,
        selector: u32,
        ival: *const c_void,
        isize: usize,
        oval: *mut c_void,
        osize: *mut usize,
    ) -> i32;
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KeyDataVer {
    pub major: u8,
    pub minor: u8,
    pub build: u8,
    pub reserved: u8,
    pub release: u16,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct PLimitData {
    pub version: u16,
    pub length: u16,
    pub cpu_p_limit: u32,
    pub gpu_p_limit: u32,
    pub mem_p_limit: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct KeyInfo {
    pub data_size: u32,
    pub data_type: u32,
    pub data_attributes: u8,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct KeyData {
    pub key: u32,
    pub vers: KeyDataVer,
    pub p_limit_data: PLimitData,
    pub key_info: KeyInfo,
    pub result: u8,
    pub status: u8,
    pub data8: u8,
    pub data32: u32,
    pub bytes: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct TempSensorVal {
    pub sensor_code_name: String,
    pub sensor_data: Vec<u8>,
}

pub struct AppleSMCClient {
    rust_client_connection: u32,
    sensor_names_map: HashMap<u32, KeyInfo>,
}

//the core class per se, client that is exported to main.rs and used to generate a stream of data from temp sensors
impl AppleSMCClient {
    // creating a new client by finding the “AppleSMCKeysEndpoint” service and opening a user connection to it
    pub fn new() -> WithError<Self> {
        let mut rust_client_connection = 0;

        for (device, name) in IOServiceIterator::new("AppleSMC")? {
            if name == "AppleSMCKeysEndpoint" {
                //opening user client connection,
                // return code generated by IOService::newUserClient, 0 in case of success
                let io_service_return_code = unsafe {
                    IOServiceOpen(device, mach_task_self(), 0, &mut rust_client_connection)
                };
                if io_service_return_code != 0 {
                    return Err(format!("IOServiceOpen: {}", io_service_return_code).into());
                }
            }
        }

        Ok(Self {
            rust_client_connection,
            sensor_names_map: HashMap::new(),
        })
    }

    // a "read" call on the SMC service using IOConnectCallStructMethod
    fn read(&self, input: &KeyData) -> WithError<KeyData> {
        //telling IOKit how big is input buffer is and how big is output buffer
        //if the call succeeds, kernel/temp sensors will write back oval
        let ival = input as *const _ as _;
        let ilen = size_of::<KeyData>();
        let mut oval = KeyData::default();
        let mut olen = size_of::<KeyData>();
        // selector 2 means "read value for key"
        let response_code = unsafe {
            IOConnectCallStructMethod(
                self.rust_client_connection,
                2,
                ival,
                ilen,
                &mut oval as *mut _ as _,
                &mut olen,
            )
        };

        if response_code != 0 {
            // println!("{:?}", input);
            return Err(format!("IOConnectCallStructMethod: {}", response_code).into());
        }

        if oval.result == 132 {
            return Err("SMC key not found".into());
        }
        // any non 0 result is some error
        if oval.result != 0 {
            return Err(format!("SMC error: {}", oval.result).into());
        }

        Ok(oval)
    }

    // looking up the SMC key name (FourCC) at the given index...
    // returns the 4-character key as a Rust String
    //Wikipedia:
    //"A FourCC ("four-character code") is a sequence of four bytes (typically ASCII) used to uniquely identify data formats."
    pub fn key_by_index(&self, index: u32) -> WithError<String> {
        let ival = KeyData {
            data8: 8,
            data32: index,
            ..Default::default()
        };
        let oval = self.read(&ival)?;
        Ok(std::str::from_utf8(&oval.key.to_be_bytes())
            .unwrap()
            .to_string())
    }
    // reading the KeyInfo (data size, type, attributes) for a rethrived 4-byte sensor code name
    // caches results in a HashMap to avoid extarcting info of the same key twice
    pub fn read_sensor_code_name_info(&mut self, sensor_code_name: &str) -> WithError<KeyInfo> {
        if sensor_code_name.len() != 4 {
            return Err("SMC key must be 4 bytes long".into());
        }

        // key is FourCC
        let key = sensor_code_name
            .bytes()
            .fold(0, |acc, x| (acc << 8) + x as u32);
        if let Some(ki) = self.sensor_names_map.get(&key) {
            // println!("cache hit for {}", key);
            return Ok(ki.clone());
        }
        //selector 9 == "read key info"
        let ival = KeyData {
            data8: 9,
            key,
            ..Default::default()
        };
        let oval = self.read(&ival)?;
        self.sensor_names_map.insert(key, oval.key_info);
        Ok(oval.key_info)
    }
    // read the raw data bytes for aselected  sensor code name
    // calls both read_sensor_code_name_info() + read()
    pub fn read_val(&mut self, sensor_code_name: &str) -> WithError<TempSensorVal> {
        let sensor_code_name = sensor_code_name.to_string();

        let key_info = self.read_sensor_code_name_info(&sensor_code_name)?;
        let key = sensor_code_name
            .bytes()
            .fold(0, |acc, x| (acc << 8) + x as u32);
        // println!("{:?}", key_info);

        let ival = KeyData {
            data8: 5,
            key,
            key_info,
            ..Default::default()
        };
        let oval = self.read(&ival)?;
        // println!("{:?}", oval.bytes);

        Ok(TempSensorVal {
            sensor_code_name,
            sensor_data: oval.bytes[0..key_info.data_size as usize].to_vec(),
        })
    }
    // read all available sensor code names by first reading the special "#KEY" entry
    // "#KEY" returns the number of keys, then function iterates over them
    pub fn read_all_sensors(&mut self) -> WithError<Vec<String>> {
        let val = self.read_val("#KEY")?;
        let val = u32::from_be_bytes(val.sensor_data[0..4].try_into().unwrap());

        let mut sensor_code_names = Vec::new();
        for i in 0..val {
            let sensor_code_name = self.key_by_index(i)?;
            let val = self.read_val(&sensor_code_name);
            if val.is_err() {
                continue;
            }

            let val = val.unwrap();
            sensor_code_names.push(val.sensor_code_name);
        }

        Ok(sensor_code_names)
    }
}

impl Drop for AppleSMCClient {
    fn drop(&mut self) {
        unsafe {
            IOServiceClose(self.rust_client_connection);
        }
    }
}
