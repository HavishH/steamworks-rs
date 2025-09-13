use super::*;

/// Access to the Steam Game Coordinator interface
pub struct GameCoordinator {
    pub(crate) gc: *mut sys::ISteamGameCoordinator,
    pub(crate) _inner: Arc<Inner>,
}

impl GameCoordinator {
    /// Send a message to the game coordinator. Returns the EGCResults value as sys::EGCResults
    pub fn send_message(&self, msg_type: u32, data: &[u8]) -> sys::EGCResults {
        unsafe {
            let res = sys::SteamGC_SendMessage(
                self.gc,
                msg_type,
                data.as_ptr().cast(),
                data.len() as u32,
            );
            // bindgen produced uint32 return; cast back to enum
            std::mem::transmute(res as i32)
        }
    }

    /// Check if a message is available and return its size
    pub fn is_message_available(&self) -> Option<u32> {
        unsafe {
            let mut sz: u32 = 0;
            if sys::SteamGC_IsMessageAvailable(self.gc, &mut sz) {
                Some(sz)
            } else {
                None
            }
        }
    }

    /// Retrieve a message into the provided buffer. Returns (EGCResults, bytes_written, msg_type)
    pub fn retrieve_message(&self, buf: &mut [u8]) -> (sys::EGCResults, u32, u32) {
        unsafe {
            let mut msg_type: u32 = 0;
            let mut out_sz: u32 = 0;
            let res = sys::SteamGC_RetrieveMessage(
                self.gc,
                &mut msg_type,
                buf.as_mut_ptr().cast(),
                buf.len() as u32,
                &mut out_sz,
            );
            (std::mem::transmute(res as i32), out_sz, msg_type)
        }
    }
}
