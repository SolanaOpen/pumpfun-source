//! Recovery of the memory-compare shim at 0xc2730..0xc2770.
//! Native execution uses a byte loop; SBF delegates to the VM syscall.
#![no_std]
#[cfg(target_os = "solana")]
extern "C" {
    fn sol_memcmp_(a: *const u8, b: *const u8, len: u64, result: *mut i32) -> u64;
}

/// # Safety
/// Both pointers must be readable for `len` bytes.
#[no_mangle]
#[inline(never)]
pub unsafe fn recovered_memcmp(a: *const u8, b: *const u8, len: usize) -> i32 {
    #[cfg(target_os = "solana")]
    {
        let mut result = 0;
        sol_memcmp_(a, b, len as u64, &mut result);
        result
    }
    #[cfg(not(target_os = "solana"))]
    {
        for i in 0..len {
            let difference = *a.add(i) as i32 - *b.add(i) as i32;
            if difference != 0 { return difference; }
        }
        0
    }
}
