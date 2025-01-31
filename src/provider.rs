//! ETW Providers abstraction.
//!
//! Provides an abstraction over an [ETW Provider](https://docs.microsoft.com/en-us/windows/win32/etw/about-event-tracing#providers)
use super::traits::*;
use crate::native::etw_types::EventRecord;
use crate::native::pla;
use crate::schema;
use std::sync::{Arc, RwLock};
use windows::Guid;

/// Provider module errors
#[derive(Debug)]
pub enum ProviderError {
    /// Returned whenever a provider doesn't have an associated GUID
    NoGuid,
    /// Wrapper over an internal [PlaError]
    ///
    /// [PlaError]: crate::native::pla::PlaError
    ComProvider(pla::PlaError),
    /// Wrapper over an standard IO Error
    IoError(std::io::Error),
}

impl LastOsError<ProviderError> for ProviderError {}

impl From<std::io::Error> for ProviderError {
    fn from(err: std::io::Error) -> Self {
        ProviderError::IoError(err)
    }
}

impl From<pla::PlaError> for ProviderError {
    fn from(err: pla::PlaError) -> Self {
        ProviderError::ComProvider(err)
    }
}

type ProviderResult<T> = Result<T, ProviderError>;

/// Kernel Providers module
///
/// Provides an easy way to create a Kernel Provider. Multiple providers are pre-created statically with
/// their appropriate GUID and flags
/// Credits: [KrabsETW::kernel_providers](https://github.com/microsoft/krabsetw/blob/master/krabs/krabs/kernel_providers.hpp)
// TODO: Extremely Verbose and cumbersome, think a way to do this in a more clean way
#[allow(dead_code)]
pub mod kernel_providers {
    use super::Guid;

    /// List of Kernel Providers GUIDs
    ///
    /// Credits: [KrabsETW::kernel_guids](https://github.com/microsoft/krabsetw/blob/master/krabs/krabs/kernel_guids.hpp)
    pub mod kernel_guids {
        pub const ALPC_GUID: &str = "45d8cccd-539f-4b72-a8b7-5c683142609a";
        pub const POWER_GUID: &str = "e43445e0-0903-48c3-b878-ff0fccebdd04";
        pub const DEBUG_GUID: &str = "13976d09-a327-438c-950b-7f03192815c7";
        pub const TCP_IP_GUID: &str = "9a280ac0-c8e0-11d1-84e2-00c04fb998a2";
        pub const UDP_IP_GUID: &str = "bf3a50c5-a9c9-4988-a005-2df0b7c80f80";
        pub const THREAD_GUID: &str = "3d6fa8d1-fe05-11d0-9dda-00c04fd7ba7c";
        pub const DISK_IO_GUID: &str = "3d6fa8d4-fe05-11d0-9dda-00c04fd7ba7c";
        pub const FILE_IO_GUID: &str = "90cbdc39-4a3e-11d1-84f4-0000f80464e3";
        pub const PROCESS_GUID: &str = "3d6fa8d0-fe05-11d0-9dda-00c04fd7ba7c";
        pub const REGISTRY_GUID: &str = "AE53722E-C863-11d2-8659-00C04FA321A1";
        pub const SPLIT_IO_GUID: &str = "d837ca92-12b9-44a5-ad6a-3a65b3578aa8";
        pub const OB_TRACE_GUID: &str = "89497f50-effe-4440-8cf2-ce6b1cdcaca7";
        pub const UMS_EVENT_GUID: &str = "9aec974b-5b8e-4118-9b92-3186d8002ce5";
        pub const PERF_INFO_GUID: &str = "ce1dbfb4-137e-4da6-87b0-3f59aa102cbc";
        pub const PAGE_FAULT_GUID: &str = "3d6fa8d3-fe05-11d0-9dda-00c04fd7ba7c";
        pub const IMAGE_LOAD_GUID: &str = "2cb15d1d-5fc1-11d2-abe1-00a0c911f518";
        pub const POOL_TRACE_GUID: &str = "0268a8b6-74fd-4302-9dd0-6e8f1795c0cf";
        pub const LOST_EVENT_GUID: &str = "6a399ae0-4bc6-4de9-870b-3657f8947e7e";
        pub const STACK_WALK_GUID: &str = "def2fe46-7bd6-4b80-bd94-f57fe20d0ce3";
        pub const EVENT_TRACE_GUID: &str = "68fdd900-4a3e-11d1-84f4-0000f80464e3";
        pub const MMCSS_TRACE_GUID: &str = "f8f10121-b617-4a56-868b-9df1b27fe32c";
        pub const SYSTEM_TRACE_GUID: &str = "9e814aad-3204-11d2-9a82-006008a86939";
        pub const EVENT_TRACE_CONFIG_GUID: &str = "01853a65-418f-4f36-aefc-dc0f1d2fd235";
    }

    /// List of Kernel Providers flags
    ///
    /// More info: [EVENT_TRACE_PROPERTIES->EnableFlags](https://docs.microsoft.com/en-us/windows/win32/api/evntrace/ns-evntrace-event_trace_properties)
    pub mod kernel_flags {
        pub const EVENT_TRACE_FLAG_PROCESS: u32 = 0x00000001;
        pub const EVENT_TRACE_FLAG_THREAD: u32 = 0x00000002;
        pub const EVENT_TRACE_FLAG_IMAGE_LOAD: u32 = 0x00000004;
        pub const EVENT_TRACE_FLAG_PROCESS_COUNTERS: u32 = 0x00000008;
        pub const EVENT_TRACE_FLAG_CSWITCH: u32 = 0x00000010;
        pub const EVENT_TRACE_FLAG_DPC: u32 = 0x00000020;
        pub const EVENT_TRACE_FLAG_INTERRUPT: u32 = 0x00000040;
        pub const EVENT_TRACE_FLAG_SYSTEMCALL: u32 = 0x00000080;
        pub const EVENT_TRACE_FLAG_DISK_IO: u32 = 0x00000100;
        pub const EVENT_TRACE_FLAG_DISK_FILE_IO: u32 = 0x00000200;
        pub const EVENT_TRACE_FLAG_DISK_IO_INIT: u32 = 0x00000400;
        pub const EVENT_TRACE_FLAG_DISPATCHER: u32 = 0x00000800;
        pub const EVENT_TRACE_FLAG_MEMORY_PAGE_FAULTS: u32 = 0x00001000;
        pub const EVENT_TRACE_FLAG_MEMORY_HARD_FAULTS: u32 = 0x00002000;
        pub const EVENT_TRACE_FLAG_VIRTUAL_ALLOC: u32 = 0x00004000;
        pub const EVENT_TRACE_FLAG_VAMAP: u32 = 0x00008000;
        pub const EVENT_TRACE_FLAG_NETWORK_TCPIP: u32 = 0x00010000;
        pub const EVENT_TRACE_FLAG_REGISTRY: u32 = 0x00020000;
        pub const EVENT_TRACE_FLAG_DBGPRINT: u32 = 0x00040000;
        pub const EVENT_TRACE_FLAG_ALPC: u32 = 0x00100000;
        pub const EVENT_TRACE_FLAG_SPLIT_IO: u32 = 0x00200000;
        pub const EVENT_TRACE_FLAG_DRIVER: u32 = 0x00800000;
        pub const EVENT_TRACE_FLAG_PROFILE: u32 = 0x01000000;
        pub const EVENT_TRACE_FLAG_FILE_IO: u32 = 0x02000000;
        pub const EVENT_TRACE_FLAG_FILE_IO_INIT: u32 = 0x04000000;
    }

    /// Represents a Kernel Provider structure which can be used to create a Kernel Provider
    pub struct KernelProvider {
        /// Kernel Provider GUID
        pub guid: Guid,
        /// Kernel Provider Flags
        pub flags: u32,
    }

    impl KernelProvider {
        /// Use the `new` function to create a Kernel Provider which can be then tied into a Provider
        pub fn new(guid: &str, flags: u32) -> KernelProvider {
            KernelProvider {
                guid: Guid::from(guid),
                flags,
            }
        }
    }

    lazy_static! {
        /// Represents the VirtualAlloc Kernel Provider
        pub static ref VIRTUAL_ALLOC_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PAGE_FAULT_GUID,
            kernel_flags::EVENT_TRACE_FLAG_VIRTUAL_ALLOC
        );
        /// Represents the VA Map Kernel Provider
        pub static ref VAMAP_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::FILE_IO_GUID, kernel_flags::EVENT_TRACE_FLAG_VAMAP);
        /// Represents the Thread Kernel Provider
        pub static ref THREAD_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::THREAD_GUID, kernel_flags::EVENT_TRACE_FLAG_THREAD);
        /// Represents the Split IO Kernel Provider
        pub static ref SPLIT_IO_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::SPLIT_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_SPLIT_IO
        );
        /// Represents the SystemCall Kernel Provider
        pub static ref SYSTEM_CALL_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PERF_INFO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_SYSTEMCALL
        );
        /// Represents the Registry Kernel Provider
        pub static ref REGISTRY_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::REGISTRY_GUID,
            kernel_flags::EVENT_TRACE_FLAG_REGISTRY
        );
        /// Represents the Profile Kernel Provider
        pub static ref PROFILE_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PERF_INFO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_PROFILE
        );
        /// Represents the Process Counter Kernel Provider
        pub static ref PROCESS_COUNTER_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PROCESS_GUID,
            kernel_flags::EVENT_TRACE_FLAG_PROCESS_COUNTERS
        );
        /// Represents the Process Kernel Provider
        pub static ref PROCESS_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PROCESS_GUID,
            kernel_flags::EVENT_TRACE_FLAG_PROCESS
        );
        /// Represents the TCP-IP Kernel Provider
        pub static ref TCP_IP_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::TCP_IP_GUID,
            kernel_flags::EVENT_TRACE_FLAG_NETWORK_TCPIP
        );
        /// Represents the Memory Page Fault Kernel Provider
        pub static ref MEMORY_PAGE_FAULT_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PAGE_FAULT_GUID,
            kernel_flags::EVENT_TRACE_FLAG_MEMORY_PAGE_FAULTS
        );
        /// Represents the Memory Hard Fault Kernel Provider
        pub static ref MEMORY_HARD_FAULT_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PAGE_FAULT_GUID,
            kernel_flags::EVENT_TRACE_FLAG_MEMORY_HARD_FAULTS
        );
        /// Represents the Interrupt Kernel Provider
        pub static ref INTERRUPT_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::PERF_INFO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_INTERRUPT
        );
        /// Represents the Driver Kernel Provider
        pub static ref DRIVER_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::DISK_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_DISK_IO
        );
        /// Represents the DPC Kernel Provider
        pub static ref DPC_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::PERF_INFO_GUID, kernel_flags::EVENT_TRACE_FLAG_DPC);
        /// Represents the Image Load Kernel Provider
        pub static ref IMAGE_LOAD_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::IMAGE_LOAD_GUID,
            kernel_flags::EVENT_TRACE_FLAG_IMAGE_LOAD
        );
        /// Represents the Thread Dispatcher Kernel Provider
        pub static ref THREAD_DISPATCHER_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::THREAD_GUID,
            kernel_flags::EVENT_TRACE_FLAG_DISPATCHER
        );
        /// Represents the File Init IO Kernel Provider
        pub static ref FILE_INIT_IO_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::FILE_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_FILE_IO_INIT
        );
        /// Represents the File IO Kernel Provider
        pub static ref FILE_IO_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::FILE_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_FILE_IO
        );
        /// Represents the Disk IO Init Kernel Provider
        pub static ref DISK_IO_INIT_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::DISK_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_DISK_IO_INIT
        );
        /// Represents the Disk IO Kernel Provider
        pub static ref DISK_IO_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::DISK_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_DISK_IO
        );
        /// Represents the Disk File IO Kernel Provider
        pub static ref DISK_FILE_IO_PROVIDER: KernelProvider = KernelProvider::new(
            kernel_guids::DISK_IO_GUID,
            kernel_flags::EVENT_TRACE_FLAG_DISK_FILE_IO
        );
        /// Represents the Dbg Pring Kernel Provider
        pub static ref DEBUG_PRINT_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::DEBUG_GUID, kernel_flags::EVENT_TRACE_FLAG_DBGPRINT);
        /// Represents the Context Swtich Kernel Provider
        pub static ref CONTEXT_SWITCH_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::THREAD_GUID, kernel_flags::EVENT_TRACE_FLAG_CSWITCH);
        /// Represents the ALPC Kernel Provider
        pub static ref ALPC_PROVIDER: KernelProvider =
            KernelProvider::new(kernel_guids::ALPC_GUID, kernel_flags::EVENT_TRACE_FLAG_ALPC);
    }
}

/// Main Provider structure
pub struct Provider {
    /// Option that represents a Provider GUID
    pub guid: Option<Guid>,
    /// Provider Any keyword
    pub any: u64,
    /// Provider All keyword
    pub all: u64,
    /// Provider level flag
    pub level: u8,
    /// Provider trace flags
    pub trace_flags: u32,
    /// Provider kernel flags, only apply to KernelProvider
    pub flags: u32, // Only applies to KernelProviders
    // perfinfo
    callbacks: Arc<
        RwLock<
            Vec<Box<dyn FnMut(EventRecord, &mut schema::SchemaLocator) + Send + Sync + 'static>>,
        >,
    >,
    // filters: RwLock<Vec<F>>,
}

impl std::fmt::Debug for Provider {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Provider {
    /// Use the `new` function to create a Provider builder
    ///
    /// This function will create a by-default provider which can be tweaked afterwards
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new();
    /// ```
    pub fn new() -> Self {
        Provider {
            guid: None,
            any: 0,
            all: 0,
            level: 5,
            trace_flags: 0,
            flags: 0,
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Use the `new_kernel` function to create a Provider builder wrapping a Kernel Provider
    ///
    /// # Arguments
    /// * `kernel_provider` - Reference to a KernelProvider which will be tied to the Provider struct
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::kernel(&kernel_providers::IMAGE_LOAD_PROVIDER);
    /// ```
    pub fn kernel(kernel_provider: &kernel_providers::KernelProvider) -> Self {
        Provider {
            guid: Some(kernel_provider.guid),
            any: 0,
            all: 0,
            level: 5,
            trace_flags: 0,
            flags: kernel_provider.flags,
            callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Use the `by_guid` function to bind a GUID with a Provider
    ///
    /// # Arguments
    /// * `guid` - A string representation of the GUID, without curly braces, that is being binded to the Provider
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new().by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716");
    /// ```
    pub fn by_guid(mut self, guid: &str) -> Self {
        self.guid = Some(Guid::from(guid));
        self
    }

    /// Use the `by_name` function to bind a GUID with a Provider
    ///
    /// This function will look for the Provider GUID by means of the [ITraceDataProviderCollection](https://docs.microsoft.com/en-us/windows/win32/api/pla/nn-pla-itracedataprovidercollection)
    /// interface.
    ///
    /// # Remark
    /// This function is considerably slow, prefer using the `by_guid` function when possible
    ///
    /// # Arguments
    /// * `name` - Provider name to find
    ///
    /// # Safety Note
    /// This function won't fail if the Provider GUID can't be found, it will log the event and set the Guid field to None. This behavior might change in the future
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new().by_name(String::from("Microsoft-Windows-WinINet"));
    /// ```
    pub fn by_name(mut self, name: String) -> Self {
        unsafe {
            match pla::get_provider_guid(&name) {
                Ok(res) => self.guid = Some(res),
                Err(err) => {
                    println!("{:?}", err);
                    self.guid = None;
                }
            }
        }
        self
    }

    /// Use the `any` function to set the `any` flag in the Provider instance
    /// [More info](https://docs.microsoft.com/en-us/message-analyzer/system-etw-provider-event-keyword-level-settings#filtering-with-system-etw-provider-event-keywords-and-levels)
    ///
    /// # Arguments
    /// * `any` - Any flag value to set
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new().any(0xf0010000000003ff);
    /// ```   
    pub fn any(mut self, any: u64) -> Self {
        self.any = any;
        self
    }

    /// Use the `all` function to set the `all` flag in the Provider instance
    /// [More info](https://docs.microsoft.com/en-us/message-analyzer/system-etw-provider-event-keyword-level-settings#filtering-with-system-etw-provider-event-keywords-and-levels)
    ///
    /// # Arguments
    /// * `all` - All flag value to set
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new().all(0x4000000000000000);
    /// ```
    pub fn all(mut self, all: u64) -> Self {
        self.all = all;
        self
    }

    /// Use the `level` function to set the `level` flag in the Provider instance
    ///
    /// # Arguments
    /// * `level` - Level flag value to set
    ///
    /// # Example
    /// ```rust
    /// // LogAlways (0x0)
    /// // Critical (0x1)
    /// // Error (0x2)
    /// // Warning (0x3)
    /// // Information (0x4)
    /// // Verbose (0x5)
    /// let my_provider = Provider::new().level(0x5);
    /// ```
    pub fn level(mut self, level: u8) -> Self {
        self.level = level;
        self
    }

    /// Use the `trace_flags` function to set the `trace_flags` flag in the Provider instance
    /// [More info](https://docs.microsoft.com/en-us/windows-hardware/drivers/devtest/trace-flags)
    ///
    /// # Arguments
    /// * `trace_flags` - TraceFlags value to set
    ///
    /// # Example
    /// ```rust
    /// let my_provider = Provider::new().trace_flags(0x1);
    /// ```
    pub fn trace_flags(mut self, trace_flag: u32) -> Self {
        self.trace_flags = trace_flag;
        self
    }

    /// Use the `add_callback` function to add a callback function that will be called when the Provider generates an Event
    ///
    /// # Arguments
    /// * `callback` - Callback to add
    ///
    /// # Remarks
    /// The [SchemaLocator] has to be mutable because whenever we obtain a new Schema it will be saved
    /// into the [SchemaLocator] instance cache
    ///
    /// # Example
    /// ```rust
    /// Provider::new().add_callback(|record: EventRecord, schema_locator: &mut SchemaLocator| {
    ///     // Handle Event
    /// });
    /// ```
    ///
    /// [SchemaLocator]: crate::schema::SchemaLocator
    pub fn add_callback<T>(self, callback: T) -> Self
    where
        T: FnMut(EventRecord, &mut schema::SchemaLocator) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.write() {
            callbacks.push(Box::new(callback));
        }
        self
    }

    /*
    pub fn add_filter(&mut self) -> ProviderResult<()> {
        if let Ok(mut filters) = self.callbacks.write() {
            self.filters.push(callback_info);

            return Ok(());
        }
        Ok(())
    }
     */

    /// Use the `build` function to build the provider
    ///
    /// # Safety Note
    /// This function might return an [ProviderError::NoGuid] if the GUID is not set in the Provider struct
    ///
    /// # Example
    /// ```rust
    /// Provider::new()
    ///   .by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")
    ///   .add_callback(process_callback)
    ///   .build()?
    /// ```
    // TODO: should we check if callbacks is empty ???
    pub fn build(self) -> ProviderResult<Self> {
        if self.guid.is_none() {
            return Err(ProviderError::NoGuid);
        }
        Ok(self)
    }

    pub(crate) fn on_event(&self, record: EventRecord, locator: &mut schema::SchemaLocator) {
        // Has to be mutable because the SchemaLocator will be mutated when locating the schema
        // within the cb creating a clone of the whole SchemaLocator HashMap doesn't
        // sound like a plan still needs to think more about this thou...
        // Could we locate the schema before calling the callback???
        if let Ok(mut callbacks) = self.callbacks.write() {
            callbacks.iter_mut().for_each(|cb| cb(record, locator))
        }
    }
}

#[cfg(test)]
mod test {
    use super::kernel_providers::kernel_flags::*;
    use super::kernel_providers::kernel_guids::*;
    use super::kernel_providers::*;
    use super::*;

    #[test]
    fn test_set_guid() {
        let prov = Provider::new().by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716");
        assert_eq!(true, prov.guid.is_some());
    }

    #[test]
    fn test_set_guid_value() {
        let prov = Provider::new().by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716");
        assert_eq!(
            Guid::from("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"),
            prov.guid.unwrap()
        );
    }

    #[test]
    fn test_set_level() {
        let prov = Provider::new().level(1);
        assert_eq!(1, prov.level);
    }

    #[test]
    fn test_set_any() {
        let prov = Provider::new().any(0x1993);
        assert_eq!(0x1993, prov.any);
    }

    #[test]
    fn test_set_all() {
        let prov = Provider::new().all(0x1302);
        assert_eq!(0x1302, prov.all);
    }

    #[test]
    fn test_set_trace_flags() {
        let prov = Provider::new().trace_flags(100);
        assert_eq!(100, prov.trace_flags);
    }

    #[test]
    fn test_set_callback() {
        let prov = Provider::new().add_callback(|_x, _y| {});
        assert_eq!(1, prov.callbacks.read().unwrap().len());
    }

    #[test]
    fn test_set_multiple_callbacks() {
        let prov = Provider::new()
            .add_callback(|_x, _y| {})
            .add_callback(|_x, _y| {})
            .add_callback(|_x, _y| {});
        assert_eq!(3, prov.callbacks.read().unwrap().len());
    }

    #[test]
    fn test_builder_fail_no_guid() {
        let prov = Provider::new().build();
        assert_eq!(true, prov.is_err());
    }

    #[test]
    fn test_builder_return_ok() {
        let prov = Provider::new()
            .by_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")
            .build();
        assert_eq!(true, prov.is_ok());
    }

    #[test]
    fn test_kernel_provider_struct() {
        let kernel_provider = KernelProvider::new("D396B546-287D-4712-A7F5-8BE226A8C643", 0x10000);

        assert_eq!(0x10000, kernel_provider.flags);
        assert_eq!(
            Guid::from("D396B546-287D-4712-A7F5-8BE226A8C643"),
            kernel_provider.guid
        );
    }

    #[test]
    fn test_kernel_provider_is_binded_to_provider() {
        let kernel_provider = Provider::kernel(&IMAGE_LOAD_PROVIDER).build();

        assert_eq!(true, kernel_provider.is_ok());

        let kernel_provider = kernel_provider.unwrap();

        assert_eq!(EVENT_TRACE_FLAG_IMAGE_LOAD, kernel_provider.flags);
        assert_eq!(true, kernel_provider.guid.is_some());
        assert_eq!(Guid::from(IMAGE_LOAD_GUID), kernel_provider.guid.unwrap());
    }
}
