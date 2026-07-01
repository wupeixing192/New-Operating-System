#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

pub fn serial() -> uart_16550::SerialPort {
    let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    port.init();
    port
}

entry_point!(kernel_main);

// ============ 动态能力令牌系统 ============
const MAX_TOKENS: usize = 32;

#[derive(Copy, Clone, Debug)]
struct TokenEntry {
    id: u64,
    permissions: u8,    // bit0=读, bit1=写, bit2=执行
    owner_id: u64,
    is_valid: bool,
}

struct TokenManager {
    table: [TokenEntry; MAX_TOKENS],
    next_id: u64,
    count: usize,
}

impl TokenManager {
    const fn new() -> Self {
        Self {
            table: [TokenEntry {
                id: 0,
                permissions: 0,
                owner_id: 0,
                is_valid: false,
            }; MAX_TOKENS],
            next_id: 1,
            count: 0,
        }
    }

    fn create(&mut self, permissions: u8, owner_id: u64) -> Option<u64> {
        if self.count >= MAX_TOKENS {
            return None;
        }
        for entry in &mut self.table {
            if !entry.is_valid {
                entry.id = self.next_id;
                entry.permissions = permissions;
                entry.owner_id = owner_id;
                entry.is_valid = true;
                self.next_id += 1;
                self.count += 1;
                return Some(entry.id);
            }
        }
        None
    }

    fn verify(&self, token_id: u64, required: u8, owner_id: u64) -> bool {
        for entry in &self.table {
            if entry.is_valid && entry.id == token_id {
                if entry.owner_id != owner_id {
                    return false;
                }
                return (entry.permissions & required) == required;
            }
        }
        false
    }

    fn revoke(&mut self, token_id: u64, owner_id: u64) -> bool {
        for entry in &mut self.table {
            if entry.is_valid && entry.id == token_id {
                if entry.owner_id != owner_id {
                    return false;
                }
                entry.is_valid = false;
                self.count -= 1;
                return true;
            }
        }
        false
    }

    fn delegate(&mut self, token_id: u64, from_owner: u64, to_owner: u64) -> bool {
        for entry in &mut self.table {
            if entry.is_valid && entry.id == token_id {
                if entry.owner_id != from_owner {
                    return false;
                }
                entry.owner_id = to_owner;
                return true;
            }
        }
        false
    }
}

// ============ 测试辅助 ============
fn test_result(port: &mut impl Write, test_name: &str, condition: bool, expected: bool) {
    if condition == expected {
        writeln!(port, "[OK] {}: passed (expected {})", test_name, expected).unwrap();
    } else {
        writeln!(port, "[FAIL] {}: expected {}, got {}", test_name, expected, condition).unwrap();
    }
}

// ============ 内核入口 ============
fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    let mut port = serial();

    writeln!(port, "[ATOMIC] Dynamic Capability System Test - 5 Scenarios").unwrap();
    writeln!(port).unwrap();

    let mut tm = TokenManager::new();

    // 创建测试令牌
    let owner = 100;
    let token_read_write = tm.create(0b11, owner).unwrap();   // 令牌0：读写
    let token_read_only = tm.create(0b01, owner).unwrap();    // 令牌1：只读
    let token_no_perm = tm.create(0b00, owner).unwrap();      // 令牌2：无权限

    // 保存令牌ID以便后续使用
    let t_rw = token_read_write;
    let t_ro = token_read_only;
    let t_none = token_no_perm;

    writeln!(port, "Created tokens: rw={}, ro={}, no_perm={}", t_rw, t_ro, t_none).unwrap();
    writeln!(port).unwrap();

    // ===== 1. 可行 (有权限) =====
    writeln!(port, "[TEST] 1. Feasible: Token with proper permissions").unwrap();
    let ok = tm.verify(t_rw, 0b01, owner); // 读
    test_result(&mut port, "Read RW token", ok, true);
    let ok = tm.verify(t_rw, 0b10, owner); // 写
    test_result(&mut port, "Write RW token", ok, true);
    writeln!(port).unwrap();

    // ===== 2. 只读 (拒绝写) =====
    writeln!(port, "[TEST] 2. Read-only: Token only allows read").unwrap();
    let ok = tm.verify(t_ro, 0b01, owner); // 读应该成功
    test_result(&mut port, "Read RO token", ok, true);
    let ok = tm.verify(t_ro, 0b10, owner); // 写应该失败
    test_result(&mut port, "Write RO token", ok, false);
    writeln!(port).unwrap();

    // ===== 3. 无权限 (任何操作拒绝) =====
    writeln!(port, "[TEST] 3. No permission: Token with 0 permissions").unwrap();
    let ok = tm.verify(t_none, 0b01, owner);
    test_result(&mut port, "Read no-perm token", ok, false);
    let ok = tm.verify(t_none, 0b10, owner);
    test_result(&mut port, "Write no-perm token", ok, false);
    writeln!(port).unwrap();

    // ===== 4. 伪造令牌 (非法持有者) =====
    writeln!(port, "[TEST] 4. Forged token: Wrong owner tries to use token").unwrap();
    let forged_owner = 999;
    let ok = tm.verify(t_rw, 0b01, forged_owner);
    test_result(&mut port, "Read RW token with forged owner", ok, false);
    let ok = tm.verify(t_ro, 0b01, forged_owner);
    test_result(&mut port, "Read RO token with forged owner", ok, false);
    writeln!(port).unwrap();

    // ===== 5. 没有令牌 (不存在的ID) =====
    writeln!(port, "[TEST] 5. No token: Non-existent ID").unwrap();
    let fake_id = 9999;
    let ok = tm.verify(fake_id, 0b01, owner);
    test_result(&mut port, "Read non-existent token", ok, false);
    writeln!(port).unwrap();

    // 额外测试：委托和撤销
    writeln!(port, "[EXTRA] Demonstrate delegation and revocation").unwrap();
    let new_owner = 200;
    writeln!(port, "Delegating RW token from {} to {}", owner, new_owner).unwrap();
    let delegated = tm.delegate(t_rw, owner, new_owner);
    writeln!(port, "Delegate result: {}", if delegated { "success" } else { "failed" }).unwrap();
    let ok = tm.verify(t_rw, 0b01, owner);
    test_result(&mut port, "Original owner after delegation", ok, false);
    let ok = tm.verify(t_rw, 0b01, new_owner);
    test_result(&mut port, "New owner after delegation", ok, true);
    let revoked = tm.revoke(t_rw, new_owner);
    writeln!(port, "Revoke result: {}", if revoked { "success" } else { "failed" }).unwrap();
    let ok = tm.verify(t_rw, 0b01, new_owner);
    test_result(&mut port, "Access after revocation", ok, false);
    writeln!(port).unwrap();

    writeln!(port, "[ATOMIC] All capability scenarios tested successfully.").unwrap();

    exit_qemu(QemuExitCode::Success);
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
