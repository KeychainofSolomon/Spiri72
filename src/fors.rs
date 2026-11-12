//! The lot: bytes from the operating system's own cryptographic source.
//! Nothing here seeds, mixes or falls back; if the system cannot give a lot,
//! the caller is refused.

#[cfg(unix)]
fn fill(buf: &mut [u8]) -> Result<(), String> {
    use std::io::Read;
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(buf))
        .map_err(|e| format!("the system gave no lot: {e}"))
}

#[cfg(windows)]
fn fill(buf: &mut [u8]) -> Result<(), String> {
    use std::ffi::c_void;
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x0000_0002;
    #[link(name = "bcrypt")]
    extern "system" {
        fn BCryptGenRandom(alg: *mut c_void, buf: *mut u8, len: u32, flags: u32) -> i32;
    }
    for chunk in buf.chunks_mut(u32::MAX as usize) {
        // SAFETY: the pointer and length describe a live, writable slice;
        // a null algorithm handle is allowed with the system-preferred flag.
        let status = unsafe {
            BCryptGenRandom(
                std::ptr::null_mut(),
                chunk.as_mut_ptr(),
                chunk.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            )
        };
        if status != 0 {
            return Err(format!("the system gave no lot (status {status:#x})"));
        }
    }
    Ok(())
}

pub fn bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut b = [0u8; N];
    fill(&mut b)?;
    Ok(b)
}

/// A number in [0, 1), from 53 bits of the lot.
pub fn unit() -> Result<f64, String> {
    let b = bytes::<8>()?;
    Ok((u64::from_le_bytes(b) >> 11) as f64 / (1u64 << 53) as f64)
}

pub fn hex<const N: usize>() -> Result<String, String> {
    Ok(bytes::<N>()?.iter().map(|b| format!("{b:02x}")).collect())
}
