#![no_std]
#![no_main]

extern "C" {
    fn put_char(ch: u8);
    fn yield_now();
    fn get_current_token() -> u64;
    fn use_token_raw(token_id: u64, owner_id: u64, action: u8, buf_ptr: *mut u8, len: usize) -> bool;
}

#[no_mangle]
pub extern "C" fn task_a() -> ! {
    unsafe { put_char(b'A'); }
    // 获取自己的令牌
    let token = unsafe { get_current_token() };
    // 尝试用令牌读一个字节（示例）
    let mut buf = [0u8; 1];
    let success = unsafe { use_token_raw(token, token, 0b001, buf.as_mut_ptr(), 1) };
    if success {
        unsafe { put_char(b'R'); } // 读成功
    } else {
        unsafe { put_char(b'F'); } // 失败（可能因计数超限或权限不足）
    }
    unsafe { put_char(b'\n'); }
    loop {
        unsafe { yield_now(); }
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)); }
    }
}

#[inline(never)]
pub fn task_a_wrapper() -> ! {
    task_a()
}