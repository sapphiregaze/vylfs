pub mod mount;
pub mod unmount;

use fuser::Filesystem;
use tracing::info;

pub struct VylFs;

impl Filesystem for VylFs {
    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), i32> {
        info!("Filesystem initialized");
        Ok(())
    }

    fn destroy(&mut self) {
        info!("Filesystem destroyed");
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        _parent: u64,
        _name: &std::ffi::OsStr,
        _reply: fuser::ReplyEntry,
    ) {
        info!("Lookup called");
    }
}
