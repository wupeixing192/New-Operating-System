#![no_std] // 禁用 Rust 标准库，用于裸机或内核环境，因为标准库依赖底层操作系统服务
#![no_main] // 禁用常规的 main 函数入口，需要自己定义入口点（如 _start 或 kernel_main）
#![feature(abi_x86_interrupt)] // 启用不稳定的 x86 中断处理 ABI 特性，允许使用 #[interrupt] 属性定义中断处理函数

use uefi::entry;  // 引入 UEFI 的入口点宏，用于定义 UEFI 应用或引导器的入口函数
use uefi::prelude::*;  // 引入 UEFI 的常用预导入类型和 trait，简化编程
use uefi::table::{Boot, SystemTable};  // 引入 UEFI 系统表类型（分为引导时和运行时），这里具体是引导时系统表
use uefi::table::boot::MemoryType;  // 引入 UEFI 引导服务中的内存类型枚举，用于内存分配
use uefi::Status;  // 引入 UEFI 状态码类型，表示操作结果
use uefi::mem::memory_map::MemoryMap;  // 引入 UEFI 内存映射结构体，用于获取系统内存布局
use uefi::mem::memory_map::MemoryMapOwned;
use uart_16550::SerialPort;  // 引入 16550 串口驱动，用于内核调试输出
use core::panic::PanicInfo;  // 引入核心库中的 panic 信息类型，用于定义 panic 处理函数
use core::fmt::Write;  // 引入核心库中的格式化写入 trait，用于向串口输出字符串
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};  // 引入原子类型：布尔原子和 usize 原子，用于多核同步和状态标记
use spin::Mutex;  // 引入自旋锁类型，用于实现互斥保护的共享数据结构
extern crate alloc;  // 声明外部 crate alloc，启用堆内存分配功能
use linked_list_allocator::LockedHeap;  // 引入链表分配器中的锁定堆类型，作为全局分配器
use x86_64::registers::control::{Cr3, Cr3Flags};  // 引入 x86_64 架构下的控制寄存器操作，包括 CR3
use x86_64::VirtAddr;  // 引入虚拟地址类型
use x86_64::PhysAddr;  // 引入物理地址类型
use x86_64::structures::paging::{OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB, Mapper};  // 引入页表结构及相关标志、映射器、物理帧等
use x86_64::structures::paging::FrameAllocator;  // 引入页帧分配器 trait，用于分配物理页
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};  // 引入全局描述符表 GDT 和描述符类型
use x86_64::structures::tss::TaskStateSegment;  // 引入任务状态段 TSS，用于实现特权级切换和中断栈
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};  // 引入中断描述符表 IDT 和中断栈帧、页错误错误码类型
use x86_64::instructions::tables::load_tss;  // 引入加载 TSS 的指令函数
use core::mem::MaybeUninit;  // 引入 MaybeUninit，用于未初始化的内存占位
use x86_64::registers::model_specific::Msr;  // 引入模型特定寄存器 MSR 相关操作
use core::sync::atomic::{AtomicU64, AtomicU8};  // 引入 64 位和 8 位原子类型，用于精细计数
use core::marker::PhantomData;  // 引入 PhantomData，用于在零大小类型中标记泛型参数
use core::arch::global_asm;  // 引入全局汇编宏，用于嵌入纯汇编代码
use core::arch::asm;

// 导入外部独立程序
use a_software::task_a_wrapper;  //外部程序a
use b_software::task_b_wrapper;  //外部程序b

#[cfg(feature = "multicore")]  // 条件编译属性：仅当启用了 "multicore" 特性时才编译下面的代码块
// 定义静态存储周期的字节切片，保存从外部二进制文件中读取的原始机器码
static AP_STARTUP_BIN: &[u8] = include_bytes!("ap_startup.bin");  // 编译时宏：将 "ap_startup.bin" 文件的内容完全嵌入为字节数组

/// 供用户态程序调用的串口输出函数
/// 使用内核已初始化的全局串口 DEBUG_SERIAL 实现
#[no_mangle]  // 告诉 Rust 编译器不要修改此函数的名称（禁用名称修饰），以便 C 语言或外部链接能够按原名称找到此函数
pub extern "C" fn put_char(ch: u8) {  // 声明为公开函数，使用 C 语言的调用约定（extern "C"），参数为 u8 类型，无返回值（-> () 可省略）
    // 直接使用 DEBUG_SERIAL 锁输出字符（原有注释）
    // 调用 write! 宏，将字符格式化为字符串输出到串口设备；DEBUG_SERIAL.lock() 获取互斥锁保护共享资源
    // 使用 `_` 忽略 write! 的返回值（Result 类型），因为此处不处理错误
    let _ = write!(DEBUG_SERIAL.lock(), "{}", ch as char);
}

#[global_allocator]
// 属性标记：将该静态变量指定为全局堆内存分配器，Rust 将使用它进行动态内存分配（如 Box、Vec 等）
static ALLOCATOR: LockedHeap = LockedHeap::empty();  // 定义静态分配器实例，类型为 LockedHeap（链表分配器加自旋锁），初始为空堆
const HEAP_SIZE: usize = 256 * 1024; // 定义堆大小常量：256 KiB
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];  // 定义静态可变字节数组作为堆内存区域，大小为 HEAP_SIZE，初始全零（mut 表示可修改，访问需 unsafe）
// 常量定义
pub const MAX_TASKS_PER_CORE: usize = 10;  // 每个 CPU 核心最多可容纳的任务（执行上下文）数量
pub const MAX_TOKENS: usize = 32;  // 系统全局最大的能力令牌数量
pub const MAX_OBJECTS: usize = 32;  // 系统全局最大的对象（资源）数量
pub const MAX_DELEGATION_DEPTH: u8 = 7;  // 委托链的最大深度，防止无限委托和栈溢出
pub const RECLAIM_BATCH_SIZE: usize = 8;  // 回收操作时批量处理的页帧个数，平衡效率与延迟
pub const MAX_CAUSAL_EVENTS: usize = 128;  // 因果事件日志队列的最大容量
pub const MAX_PHYS_MEM_GB: usize = 64;  // 系统支持的最大物理内存为 4 GB
pub const MAX_PHYS_MEM_PAGES: usize = MAX_PHYS_MEM_GB * 1024 * 1024 * 1024 / 4096;  // 计算 4 GB 物理内存对应的 4 KiB 页帧总数
pub const BITMAP_LEN: usize = MAX_PHYS_MEM_PAGES / 64;  // 位图数组的长度：每 64 个页帧对应一个 u64 位（每个位表示一页）
pub const HIGH_BASE: u64 = 0xffff_8880_0000_0000; // 非恒等映射的基址：所有物理地址将映射到 HIGH_BASE + phys_addr 的虚拟地址
static BITMAP: [AtomicU64; BITMAP_LEN] = [const { AtomicU64::new(u64::MAX) }; BITMAP_LEN];  // alloc_page 里用 CAS 循环代替直接位操作
static NEXT_FRAME: AtomicUsize = AtomicUsize::new(0);  // 下一次分配时开始搜索的位图位置，用于提高分配效率（原子操作，多核安全）
static DEBUG_SERIAL: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });  // 调试串口设备（COM1，基址 0x3F8），用互斥锁保护并发输出
static TOKEN_MANAGER: Mutex<TokenManager> = Mutex::new(TokenManager::new());  // 全局令牌管理器实例，用自旋锁包装实现多核安全访问
static mut GLOBAL_SALT: u64 = 0;  // 全局盐值，用于生成令牌的 auth_hash，确保令牌不可伪造
static ALLOC_LOCK: spin::Mutex<()> = spin::Mutex::new(());

// 定义一个宏，用于输出绿色“[  OK  ]”成功状态信息，带换行
macro_rules! status_ok {
    ($msg:expr) => {{
        // 获取调试串口的互斥锁，写入带ANSI绿色转义码的消息，忽略write!的Result
        let _ = write!(DEBUG_SERIAL.lock(), "\x1b[32m[  OK  ]\x1b[0m {}\n", $msg);
    }};
}

// 定义一个宏，用于输出红色“[FAILED]”失败状态信息，带换行
macro_rules! status_fail {
    ($msg:expr) => {{
        // 获取调试串口的互斥锁，写入带ANSI红色转义码的消息，忽略write!的Result
        let _ = write!(DEBUG_SERIAL.lock(), "\x1b[31m[FAILED]\x1b[0m {}\n", $msg);
    }};
}

// 一个经过审查的硬件通信黑盒工具
fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    // 准备 eax 和 ecx，cpuid 指令会使用这两个寄存器作为输入
    let mut eax = leaf;
    let mut ecx = subleaf;
    let mut ebx: u32;
    let mut edx: u32;
    unsafe {
        core::arch::asm!(
        // 交换 rbx 和 r10，因为 Rust 内联汇编会使用 rbx 保存局部变量，而 cpuid 会破坏 rbx
        "xchg rbx, r10",
        // 执行 CPUID 指令，返回结果在 eax, ebx, ecx, edx
        "cpuid",
        // 恢复 rbx 的原值
        "xchg rbx, r10",
        // 输入输出参数：eax 和 ecx 同时作为输入和输出（cpuid 会覆盖它们）
        inout("eax") eax,
        inout("ecx") ecx,
        // 输出：ebx 的临时保存用 r10，然后赋给 ebx 变量
        out("r10") ebx,
        // 输出：edx 直接被赋给 edx 变量
        out("edx") edx,
        // 选项：不访问内存，不修改栈，告诉编译器无需保存现场
        options(nomem, nostack)
        );
    }
    // 返回四个结果寄存器值
    (eax, ebx, ecx, edx)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统1：物理内存分配器
////////////////////////////////////////////////////////////////////////////////////////////////////

pub unsafe fn init_allocator(memory_map: &uefi::mem::memory_map::MemoryMapOwned) {
    // 遍历 UEFI 提供的内存映射条目，每个条目描述一段连续内存区域
    for desc in memory_map.entries() {
        // 1. 算出这个内存段的起始物理帧号（注意：desc.phys_start 是字节，必须除以 4096）
        let start_frame = desc.phys_start / 4096; // 字节地址转页帧号（4KiB 页）
        let page_count = desc.page_count; // 该区域包含的页数
        let is_free = matches!(desc.ty, uefi::boot::MemoryType::CONVENTIONAL); // true 表示该区域可供系统自由使用（空闲）

        // 2. 核心内层循环：一页一页去标记
        for offset in 0..page_count {
            let current_frame = start_frame + offset; // 当前页帧的绝对帧号

            // BDFL 必须加的保护：如果实机内存超过 64GB，超出部分直接 break 跳出
            if current_frame >= MAX_PHYS_MEM_PAGES as u64 {
                break; // 防止位图数组越界（位图只覆盖 MAX_PHYS_MEM_PAGES 个页帧）
            }

            // 3. 自己写下这两行换算：算出在位图数组里的下标和 bit 位
            let idx = (current_frame / 64) as usize ; // 位图数组索引：每 64 个帧使用一个 u64
            let bit = (current_frame % 64) as u32 ; // 在该 u64 中的位索引（0~63）

            // 4. 填空：根据 is_free 标记位图。如果空闲则清0，占用则置1
            if is_free {
                // 写清除位的代码（提示：用 ! 和 & 操作）
                BITMAP[idx].fetch_and(!(1 << bit), Ordering::AcqRel);
            } else {
                // 写设置位的代码（提示：用 | 操作）
                BITMAP[idx].fetch_or(1 << bit, Ordering::AcqRel);
            }
        }
    }
}

/// 从 UEFI 内存映射初始化物理内存位图
/// 将 CONVENTIONAL 类型的区域标记为空闲（bit=0），其他保留。
/// 从 UEFI 内存映射初始化物理内存位图
unsafe fn init_allocator_from_uefi_map(map: &uefi::mem::memory_map::MemoryMapOwned) {
    // 1. 全部置 1（占用）
    for i in 0..BITMAP_LEN {
        BITMAP[i].store(u64::MAX, Ordering::Release);
    }

    // 2. 遍历描述符，将 CONVENTIONAL 区域清 0（空闲）
    for desc in map.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            let start_frame = desc.phys_start / 4096;
            let page_count = desc.page_count;
            for offset in 0..page_count {
                let frame = start_frame + offset;
                if frame >= MAX_PHYS_MEM_PAGES as u64 {
                    break;
                }
                let idx = (frame / 64) as usize;
                let bit = (frame % 64) as u32;
                BITMAP[idx].fetch_and(!(1 << bit), Ordering::AcqRel);
            }
        }
    }
}

/// 后备位图：仅暴露 1MB ~ 2MB 安全区域
unsafe fn init_fallback_bitmap() {
    // 全部置 1（占用）
    for i in 0..BITMAP_LEN {
        BITMAP[i].store(u64::MAX, Ordering::Release);
    }

    // 释放 1MB 到 MAX_PHYS_MEM_PAGES 覆盖的所有帧
    let start_frame = 0x100000 / 4096;          // 1MB
    let end_frame = MAX_PHYS_MEM_PAGES as u64;  // 最大支持帧数
    for frame in start_frame..end_frame {
        let idx = (frame / 64) as usize;
        let bit = (frame % 64) as u32;
        BITMAP[idx].fetch_and(!(1 << bit), Ordering::AcqRel);
    }
}

pub unsafe fn alloc_pages(count: usize) -> Option<u64> {
    if count == 0 {
        return None;
    }
    let _lock = ALLOC_LOCK.lock(); // 获取全局锁，保证多核互斥

    let start = NEXT_FRAME.load(Ordering::Acquire);
    let mut idx = start;
    let mut consecutive = 0;
    let mut first_frame = 0;

    loop {
        if idx >= BITMAP_LEN {
            idx = 0;
        }
        let word = BITMAP[idx].load(Ordering::Acquire);
        if word != u64::MAX {
            for bit in 0..64 {
                if (word & (1 << bit)) == 0 {
                    if consecutive == 0 {
                        first_frame = idx * 64 + bit;
                    }
                    consecutive += 1;
                    if consecutive == count {
                        // 找到足够的连续页，占用它们
                        for i in 0..count {
                            let f = first_frame + i;
                            let idx2 = (f / 64) as usize;
                            let bit2 = (f % 64) as u32;
                            BITMAP[idx2].fetch_or(1 << bit2, Ordering::Release);
                        }
                        NEXT_FRAME.store(idx, Ordering::Release);
                        return Some((first_frame * 4096) as u64);
                    }
                } else {
                    consecutive = 0;
                }
            }
        } else {
            consecutive = 0;
        }
        idx += 1;
        if idx == start {
            // 遍历一圈未找到
            return None;
        }
    }
}

pub unsafe fn alloc_page() -> Option<u64> {
    alloc_pages(1)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统2：全局地址空间转换器
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct KernelFrameAllocator; // 空的帧分配器结构体，仅用作类型标记，实际功能由 alloc_page 实现

// 为 KernelFrameAllocator 实现 x86_64 库的 FrameAllocator trait（Size4KiB 表示支持 4KiB 页帧）
unsafe impl FrameAllocator<Size4KiB> for KernelFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // 因为 alloc_page 被标记为 unsafe，必须用 unsafe 块包起来
        unsafe {
            alloc_page().map(|phys| PhysFrame::from_start_address(PhysAddr::new(phys)).unwrap()) // 将物理地址转为 PhysFrame，若地址无效则 unwrap（预期不会失败）
        }
    }
}

/// 在 UEFI 1:1 映射下构建新的页表（返回新 CR3 的物理地址）
unsafe fn build_pagetable_in_uefi() -> Result<u64, &'static str> {
    let _ = write!(DEBUG_SERIAL.lock(), "build_pagetable: starting in UEFI 1:1 mode...\n"); // 输出调试信息

    // 1. 分配新 PML4（顶层页表）
    let new_pml4_phys = alloc_page().ok_or("Failed to allocate PML4 page")?; // 物理地址，出错时返回错误字符串
    let new_pml4_virt = new_pml4_phys as *mut u64; // 将物理地址视为虚拟地址指针（在 UEFI 1:1 映射下有效）
    core::ptr::write_bytes(new_pml4_virt, 0, 512); // 将整个 PML4 页（512 个 8 字节项）清零
    let _ = write!(DEBUG_SERIAL.lock(), "  New PML4 at phys {:#x}\n", new_pml4_phys); // 输出调试

    // 2. 复制旧的 PML4 中低 256 项（保持 UEFI 1:1 映射，用于 MMIO）
    let (old_cr3, _) = Cr3::read(); // 读取当前 CR3 寄存器，获得旧 PML4 的物理地址
    let old_pml4_phys = old_cr3.start_address().as_u64(); // 旧 PML4 的物理地址
    let old_pml4_virt = old_pml4_phys as *mut u64; // 转换为虚拟指针（在 1:1 映射下）
    for i in 0..256 { // 只复制低 256 项（对应虚拟地址低 47 位中的高 9 位 0-255，即 0~0x0000_FFFF_FFFF_FFFF）
        let entry = old_pml4_virt.add(i).read_volatile(); // 使用 volatile 读取，确保从内存中直接读取
        if entry != 0 { // 若该表项有效则复制
            new_pml4_virt.add(i).write_volatile(entry); // 写入新 PML4
        }
    }
    let _ = write!(DEBUG_SERIAL.lock(), "  Copied low 256 PML4 entries (1:1 mapping)\n");

    // 3. 建立 HIGH_BASE 映射（PML4 索引 = HIGH_BASE >> 39）
    let high_pml4_idx = ((HIGH_BASE >> 39) & 0x1FF) as usize; // 计算 HIGH_BASE 对应的 PML4 表项索引（9 位）
    let pdpt_phys = alloc_page().ok_or("Failed to allocate PDPT page")?; // 分配一个页用作 PDPT（页目录指针表）
    let pdpt_virt = pdpt_phys as *mut u64; // 虚拟指针
    core::ptr::write_bytes(pdpt_virt, 0, 512); // 清零 PDPT
    new_pml4_virt.add(high_pml4_idx).write_volatile(pdpt_phys | 0b11); // 写入 PML4 表项，设置存在位和读写位（0b11）
    let _ = write!(DEBUG_SERIAL.lock(), "  PDPT at phys {:#x}\n", pdpt_phys);

    // 4. 在 HIGH_BASE 下建立 4 个 PD 页（覆盖 4GB，使用 2MB 大页）
    for pd_idx in 0..4 { // 每个 PD 页管理 1GB 空间（512 个 2MB 大页），4 个共 4GB
        let pd_phys = alloc_page().ok_or("Failed to allocate PD page")?; // 分配页目录页
        let pd_virt = pd_phys as *mut u64; // 虚拟指针
        core::ptr::write_bytes(pd_virt, 0, 512); // 清零 PD 页
        pdpt_virt.add(pd_idx).write_volatile(pd_phys | 0b11); // 在 PDPT 中设置表项，指向该 PD 页

        let base_phys = (pd_idx as u64) * 0x4000_0000; // 该 PD 页对应的物理基地址（每 1GB 对齐）
        for pt_idx in 0..512 { // 每个 PD 页有 512 个表项，每个代表一个 2MB 大页
            let phys = base_phys + (pt_idx as u64) * 0x200_000; // 该大页的物理起始地址
            let huge_flag = 0x80; // x86-64 大页标志（PS 位，在页目录表项中表示 2MB 页）
            pd_virt.add(pt_idx).write_volatile(phys | 0b11 | huge_flag); // 写入 PD 表项，设置存在、读写和大页标志
        }
        let _ = write!(DEBUG_SERIAL.lock(), "  PD {} at phys {:#x} mapped 0x{:x}-0x{:x}\n",
                       pd_idx, pd_phys, base_phys, base_phys + 0x4000_0000); // 输出调试
    }

    // 5. 递归映射（PML4[511] 指向自身）
    new_pml4_virt.add(511).write_volatile(new_pml4_phys | 0b11); // 将 PML4 的最后一项（索引511）指向自身，实现递归页表，方便虚拟访问任意页表
    let _ = write!(DEBUG_SERIAL.lock(), "  Recursive mapping at PML4[511] = {:#x}\n", new_pml4_phys);

    let _ = write!(DEBUG_SERIAL.lock(), "build_pagetable: completed, new CR3 = {:#x}\n", new_pml4_phys);
    Ok(new_pml4_phys) // 返回新 PML4 的物理地址
}

/// 冻结页表：建立 `HIGH_BASE + phys` 的非恒等映射，然后切换到新 CR3
pub unsafe fn init_paging() -> Result<(), &'static str> {
    let _ = write!(DEBUG_SERIAL.lock(), "init_paging: freezing page tables...\n");

    let new_cr3_phys = build_pagetable_in_uefi()?; // 构建页表并获得新 CR3 值

    let new_frame = PhysFrame::from_start_address(PhysAddr::new(new_cr3_phys))
        .map_err(|_| "Invalid CR3 physical address")?; // 将物理地址转为 PhysFrame，失败则返回错误
    Cr3::write(new_frame, Cr3Flags::empty()); // 写入 CR3，切换页表，Cr3Flags 为空表示无额外标志

    // 替换 cpuid 调用（此处只是执行一次 cpuid 指令，用于刷新 TLB？实际上 cpuid 会修改寄存器，此处可能只是为了确保 CPU 状态更新）
    unsafe {
        core::arch::asm!("cpuid", options(nomem, nostack)); // 执行 cpuid，不访问内存，不修改栈
    }

    let _ = write!(DEBUG_SERIAL.lock(), "init_paging: switched to new CR3 = {:#x}\n", new_cr3_phys);

    // 验证映射：通过 HIGH_BASE 访问物理地址 0x1000 进行读写测试
    let test_phys = 0x1000; // 物理地址 0x1000（通常可用）
    let test_virt = HIGH_BASE + test_phys; // 计算对应的虚拟地址
    let test_ptr = test_virt as *mut u8; // 转换为可变字节指针
    unsafe { test_ptr.write_volatile(0x5A); } // 写入测试值 0x5A
    let read_val = unsafe {
        test_ptr.read_volatile()
    }; // 读回
    if read_val == 0x5A { // 验证读写一致
        let _ = write!(DEBUG_SERIAL.lock(), "init_paging: ✓ HIGH_BASE mapping verified at {:#x}\n", test_virt);
    } else {
        return Err("HIGH_BASE mapping verification failed");
    }

    // 验证 APIC（高级可编程中断控制器）MMIO 是否可访问
    let apic_test_virt = HIGH_BASE + 0xFEE00000; // APIC 默认物理基地址 0xFEE00000
    let apic_ptr = apic_test_virt as *mut u32; // 32 位寄存器指针
    let apic_ver = unsafe {
        apic_ptr.add(0x30 / 4).read_volatile()
    }; // 读取 APIC 版本寄存器（偏移 0x30）
    if apic_ver != 0 { // 版本号非零表示可读
        let _ = write!(DEBUG_SERIAL.lock(), "init_paging: ✓ APIC accessible via HIGH_BASE, version = {:#x}\n", apic_ver);
    } else {
        return Err("APIC MMIO mapping failed");
    }

    let _ = write!(DEBUG_SERIAL.lock(), "init_paging: page table frozen, CR3 sealed.\n");
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统3：能力授权引擎
////////////////////////////////////////////////////////////////////////////////////////////////////

///这是根公密钥与发布公密钥，在键值存储中使用
const ROOT_PUBLIC_KEY_DER: &[u8] = include_bytes!("../key/root-pubkey.der");

const RELEASE_CERT_DER: &[u8] = include_bytes!("../key/release-pubkey.der");

/// 内存对象，代表一段可被授权访问的内存区域
struct ObjectEntry {
    base_addr: u64, // 该内存区域的物理起始地址（在固定映射下也可视为虚拟地址）
    len: usize, // 区域长度（字节）
    perms: u8, // 权限位掩码（如读、写、执行），与令牌中的 permissions 对应
    state: u8, // 0: ACTIVE（活跃）, 1: REVOKED（已撤销）, 2: FREED（已释放）
    ref_count: AtomicUsize, // 原子引用计数，跟踪持有该对象的令牌数量，用于安全回收
}

/// 能力令牌，代表一把“钥匙”
struct TokenEntry {
    id: u64, // 令牌的唯一标识符，由 TokenManager 分配
    object_index: usize, // 指向 objects 数组中该令牌所绑定的 ObjectEntry 下标
    permissions: u8, // 该令牌授予的权限（读/写/执行等），是 ObjectEntry.perms 的子集
    owner_id: u64, // 当前持有该令牌的执行上下文 ID（可能经过委托）
    creator_id: u64, // 最初创建该令牌的上下文 ID（根创建者），用于委托链溯源
    auth_hash: u64, // 密码学校验值，由 GLOBAL_SALT 和各字段计算，防止令牌伪造
    is_valid: bool, // 令牌是否有效（撤销后置 false，不再接受验证）
    pub use_count: AtomicUsize,  // 使用计数
    pub frozen: AtomicBool,  // 是否冻结
}

/// 全局令牌管理器
pub struct TokenManager {
    table: [MaybeUninit<TokenEntry>; MAX_TOKENS],
    objects: [MaybeUninit<ObjectEntry>; MAX_OBJECTS],
    used_table: u64,      // 位图，标记令牌槽是否已使用 (1=已用)
    used_objects: u64,    // 位图，标记对象槽是否已使用 (1=已用)
    next_id: u64,
}

impl TokenManager {
    // ★ 必须是 const fn，因为静态变量需要编译时初始化
    pub const fn new() -> Self {
        Self {
            table: [const { MaybeUninit::uninit() }; MAX_TOKENS],
            objects: [const { MaybeUninit::uninit() }; MAX_OBJECTS],
            used_table: 0,
            used_objects: 0,
            next_id: 1,
        }
    }

    // compute_auth_hash 不变（无需 self）
    // compute_auth_hash 不变（无需 self）
    fn compute_auth_hash(owner: u64, perms: u8, obj_idx: usize, id: u64) -> u64 {
        // 注意：GLOBAL_SALT 现在是启动时初始化的静态变量，但在 const 上下文中不能用 unsafe
        // 所以 compute_auth_hash 不能是 const，但 new 是 const，所以 new 不能调用它。
        // 这没问题，因为 new 只初始化字段，compute_auth_hash 在运行时调用。
        let mut hash = unsafe { GLOBAL_SALT };
        hash ^= owner.wrapping_mul(0x9E3779B97F4A7C15);
        hash ^= (perms as u64).wrapping_mul(0xBF58476D1CE4E5B9);
        hash ^= (obj_idx as u64).wrapping_mul(0x94D049BB133111EB);
        hash ^= id.wrapping_mul(0x9E3779B97F4A7C15);
        hash
    }

    /// 创建新令牌
    pub fn create(&mut self, perms: u8, owner: u64, base: u64, len: usize) -> Option<u64> {
        // 查找空闲对象槽
        let obj_idx = (0..MAX_OBJECTS).find(|&i| (self.used_objects & (1 << i)) == 0)?;
        self.used_objects |= 1 << obj_idx;

        // 查找空闲令牌槽
        let tok_idx = (0..MAX_TOKENS).find(|&i| (self.used_table & (1 << i)) == 0)?;
        self.used_table |= 1 << tok_idx;

        // 写入 ObjectEntry
        let obj = ObjectEntry {
            base_addr: base,
            len,
            perms,
            state: 0,
            ref_count: AtomicUsize::new(1),
        };
        self.objects[obj_idx].write(obj);

        let id = self.next_id;
        self.next_id += 1;
        let auth_hash = Self::compute_auth_hash(owner, perms, obj_idx, id);

        let tok = TokenEntry {
            id,
            object_index: obj_idx,
            permissions: perms,
            owner_id: owner,
            creator_id: owner,
            auth_hash,
            is_valid: true,
            use_count: AtomicUsize::new(0),
            frozen: AtomicBool::new(false),
        };
        self.table[tok_idx].write(tok);

        Some(id)
    }

    /// 验证并获取令牌
    pub fn try_acquire(&mut self, token_id: u64, required: u8, owner: u64) -> bool {
        for i in 0..MAX_TOKENS {
            // 检查该槽位是否已使用
            if (self.used_table & (1 << i)) == 0 {
                continue;
            }
            // 注意：这里变量名是 token，不是 slot
            let token = unsafe { self.table[i].assume_init_mut() };
            if token.id != token_id {
                continue;
            }

            if token.frozen.load(Ordering::Acquire) {
                return false;
            }
            if !token.is_valid {
                return false;
            }
            if token.owner_id != owner {
                return false;
            }
            if (token.permissions & required) != required {
                return false;
            }

            let expected = Self::compute_auth_hash(
                token.owner_id,
                token.permissions,
                token.object_index,
                token.id,
            );
            if token.auth_hash != expected {
                return false;
            }

            let new_count = token.use_count.fetch_add(1, Ordering::Relaxed) + 1;
            if new_count > 3 {
                token.frozen.store(true, Ordering::Release);
                record_fault(owner, token_id, 0);
                return false;
            }

            unsafe {
                let obj = self.objects[token.object_index].assume_init_ref();
                obj.ref_count.fetch_add(1, Ordering::Release);
            }
            return true;
        }
        false
    }

    pub fn revoke_all(&mut self, owner_id: u64) {
        for i in 0..MAX_TOKENS {
            if (self.used_table & (1 << i)) == 0 {
                continue;
            }
            let token = unsafe { self.table[i].assume_init_mut() };
            if token.owner_id == owner_id {
                token.is_valid = false;
                unsafe {
                    let obj = self.objects[token.object_index].assume_init_mut();
                    obj.state = 1;
                }
            }
        }
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统4：时间分片调度器
////////////////////////////////////////////////////////////////////////////////////////////////////

const STACK_SIZE: usize = 4096 * 2; // 每个任务的栈大小（4 KiB）
const TIME_SLICE: usize = 100; // 每个任务每次获得的时间片长度（tick数），耗尽后触发重新调度
const QUEUE_SIZE: usize = 64; // 事件队列容量（环形缓冲区大小）

/// 任务当前的生命周期状态（AtomicU8 实现原子交换）
#[repr(u8)] // 要求内存布局与 u8 完全一致，便于原子操作和转换
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // 派生常用 trait
pub enum TaskState {
    Ready = 0, // 就绪或正在运行（调度器会轮转）
    Suspended = 1, // 挂起（调度器直接跳过，不分配 CPU）
    Terminated = 2, // 已终止（等待回收）
}

// 辅助原子操作转换函数
impl TaskState {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => TaskState::Ready,
            1 => TaskState::Suspended,
            _ => TaskState::Terminated, // 任何其他值都视为 Terminated，安全处理
        }
    }
}

#[derive(Debug, Clone, Copy)] // 事件类型，目前仅有定时器滴答
pub enum Event {
    TimerTick,
}

/// 无锁队列，用于中断与调度器间传递事件（多生产者单消费者）
pub struct AtomicQueue {
    buffer: [MaybeUninit<Event>; QUEUE_SIZE], // 环形缓冲区，MaybeUninit 避免初始化开销
    head: AtomicUsize, // 读索引（消费者）
    tail: AtomicUsize, // 写索引（生产者）
}
impl AtomicQueue {
    pub const fn const_new() -> Self {
        Self {
            buffer: [MaybeUninit::uninit(); QUEUE_SIZE], // 全未初始化
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }
    pub fn enqueue(&self, event: Event) -> bool {
        let tail = self.tail.load(Ordering::Relaxed); // 读当前尾指针（无需严格顺序）
        let next_tail = (tail + 1) % QUEUE_SIZE; // 计算下一个位置（环绕）
        if next_tail == self.head.load(Ordering::Relaxed) {
            return false;
        } // 队列已满
        unsafe {
            let ptr = self.buffer.as_ptr().add(tail) as *mut MaybeUninit<Event>;
            ptr.write(MaybeUninit::new(event)); // 写入事件
        }
        self.tail.store(next_tail, Ordering::Release); // 发布尾指针，使事件对消费者可见
        true
    }
    pub fn dequeue(&self) -> Option<Event> {
        let head = self.head.load(Ordering::Acquire); // 获取读指针，确保看到之前的写入
        if head == self.tail.load(Ordering::Relaxed) {
            return None;
        } // 队列空
        let event = unsafe {
            self.buffer[head].assume_init_read()
        }; // 读取并替换为未初始化
        self.head.store((head + 1) % QUEUE_SIZE, Ordering::Release); // 推进读指针
        Some(event)
    }
}

static EVENT_QUEUE: AtomicQueue = AtomicQueue::const_new(); // 全局事件队列实例

// --- 任务核心（使用稳定的 MaybeUninit + 数组指针初始化） ---

#[repr(C)]
pub struct TaskContext {
    pub rsp: u64,  // 只存栈指针，其他寄存器在栈上保存
}

pub struct Task {
    pub id: u64, // 全局唯一任务 ID（用户程序对此可见）
    pub init_token: u64, // 灵魂：该任务持有的初始能力令牌 ID（用于权限验证）
    pub state: AtomicU8, // 任务当前状态（原子操作）
    pub active: AtomicBool, // 是否激活（配合状态使用）
    pub stack: [u8; STACK_SIZE], // 任务栈（固定大小）
    pub ctx: MaybeUninit<TaskContext>, // 任务上下文（未初始化，由汇编填充）
}

impl Task {
    pub const fn new(id: u64, init_token: u64) -> Self {
        Self {
            id,
            init_token,
            state: AtomicU8::new(TaskState::Ready as u8),
            active: AtomicBool::new(false),
            stack: [0; STACK_SIZE],
            ctx: MaybeUninit::uninit(),
        }
    }
}

#[repr(C)] // 确保内存布局与汇编代码一致
pub struct PerCoreData {
    pub core_id: usize, // 核心编号 (BSP为0, AP为1..n)
    pub gdt: GlobalDescriptorTable, // 该核心独有的 GDT（每个核心独立）
    pub tss: TaskStateSegment, // 该核心独有的 TSS（包含独立的 IST 栈）
    pub idt: InterruptDescriptorTable, // 该核心独有的 IDT（中断门）
    pub current_task_id: AtomicUsize, // 该核心当前运行的任务 ID（数组下标）
    pub scheduler_t: AtomicUsize, // 该核心当前任务已运行的 tick 数
    pub scheduler_used: [AtomicUsize; MAX_TASKS_PER_CORE], // 该核心各任务已占用的 tick 数
    pub task_pool: [Option<Task>; MAX_TASKS_PER_CORE], // 该核心独占的任务池（每个核心独立）
    pub task_count: AtomicUsize, // 该核心当前活跃任务的数量
    pub scheduler_ctx: MaybeUninit<TaskContext>, // 调度器自身上下文（用于切换）
}

static mut CORE_DATA: [*mut PerCoreData; MAX_CORES] = [core::ptr::null_mut(); MAX_CORES]; // 每个核心的数据，MAX_CORES 应在全局定义

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1); // 全局任务 ID 分配器

/// 在指定核心创建一个新任务，并返回该任务唯一的【初始能力令牌】
/// 入口参数：core_id (目标核心编号), entry (任务入口函数)
/// 在指定核心创建一个新任务，并返回该任务唯一的【初始能力令牌】
/// 入口参数：core_id (目标核心编号), entry (任务入口函数)
pub fn add_task(core_id: usize, entry: fn() -> !) -> Option<u64> {
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();
    let _ = write!(serial, "add_task called for core {}\n", core_id);

    let core = unsafe { &mut *CORE_DATA[core_id] };

    for i in 0..MAX_TASKS_PER_CORE {
        if core.task_pool[i].is_none() {
            let task_id = NEXT_TASK_ID.fetch_add(1, Ordering::Release);
            let init_token = TOKEN_MANAGER.lock().create(0b111, task_id, 0xDEAD_BEEF, 4096)?;

            let mut task = Task::new(task_id, init_token);

            // ★ 新栈帧：只压入返回地址，栈指针指向该地址
            let stack_top = task.stack.as_ptr() as u64 + STACK_SIZE as u64;
            let rsp = stack_top - 8; // 只留一个 8 字节给返回地址
            unsafe {
                let ptr = rsp as *mut u64;
                ptr.write(entry as u64); // 返回地址，ret 会跳转到这里
            }

            // ★ TaskContext 只存栈指针
            task.ctx.write(TaskContext { rsp });

            task.state.store(TaskState::Ready as u8, Ordering::Release);
            task.active.store(true, Ordering::Release);

            core.task_pool[i] = Some(task);
            core.task_count.fetch_add(1, Ordering::Release);
            core.scheduler_used[i].store(0, Ordering::Release);
            let _ = write!(serial, "Task added at slot {}, task_count now {}\n", i, core.task_count.load(Ordering::Acquire));
            return Some(init_token);
        }
    }
    let _ = write!(serial, "No free task slot!\n");
    None
}

/// 根据【令牌 ID】查找持有该令牌的任务（用于 suspend/resume/terminate）
fn find_task_by_token(core_id: usize, token_id: u64) -> Option<&'static mut Task> {
    let core = unsafe { &mut *CORE_DATA[core_id] };
    for slot in core.task_pool.iter_mut() {
        if let Some(task) = slot {
            if task.init_token == token_id {
                return Some(task);
            }
        }
    }
    None
}

/// 暂停任务（需验证持有该任务初始令牌的合法上下文）
pub fn suspend_task(core_id: usize, token_id: u64, owner_id: u64) -> bool {
    // 1. 安全验证门：只有持有有效令牌的上下文才能暂停它
    if !TOKEN_MANAGER.lock().try_acquire(token_id, 0b001, owner_id) { // 0b001 假定为 SUSPEND 权限
        return false;
    }

    if let Some(task) = find_task_by_token(core_id, token_id) {
        task.state.store(TaskState::Suspended as u8, Ordering::Release);
        true
    } else {
        false
    }
}

/// 恢复任务
pub fn resume_task(core_id: usize, token_id: u64, owner_id: u64) -> bool {
    if !TOKEN_MANAGER.lock().try_acquire(token_id, 0b001, owner_id) {
        return false;
    }

    if let Some(task) = find_task_by_token(core_id, token_id) {
        task.state.store(TaskState::Ready as u8, Ordering::Release);
        true
    } else {
        false
    }
}

/// 关闭并销毁任务（撤销令牌并回收资源）
pub fn terminate_task(core_id: usize, token_id: u64, owner_id: u64) {
    if !TOKEN_MANAGER.lock().try_acquire(token_id, 0b001, owner_id) {
        return;
    }

    if let Some(task) = find_task_by_token(core_id, token_id) {
        task.state.store(TaskState::Terminated as u8, Ordering::Release);
        task.active.store(false, Ordering::Release);
        // 核心动作：内核强制撤销令牌持有者的所有相关令牌
        TOKEN_MANAGER.lock().revoke_all(task.id);
    }
}

// 使用汇编专用 fn，严格加上 unsafe 和 #[naked]（无 prologue/epilogue）
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(
    old_ctx: *mut TaskContext,
    new_ctx: *const TaskContext,
) {
    core::arch::naked_asm!(
        "cli",
        // 保存所有通用寄存器
        "push rax",
        "push rcx",
        "push rdx",
        "push rbx",
        "push rbp",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // 保存栈指针到 old_ctx.rsp（偏移 64）
        "mov [rcx], rsp",
        "mov rsp, [rdx]",
        // 恢复所有寄存器（逆序）
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rbp",
        "pop rbx",
        "pop rdx",
        "pop rcx",
        "pop rax",
        "sti",
        "ret",
    );
}

// 任务主动让出 CPU（由任务自己调用）
#[no_mangle]
pub extern "C" fn yield_now() {
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();
    let _ = write!(serial, "yield_now called\n");
    let core_id = CURRENT_CORE_ID.load(Ordering::Acquire);
    let core = unsafe { &mut *CORE_DATA[core_id] };
    let current = core.current_task_id.load(Ordering::Acquire);

    let used = core.scheduler_used[current].load(Ordering::Acquire) + 1;
    core.scheduler_used[current].store(used, Ordering::Release);

    // ★ 关键：如果时间片未耗尽，继续运行当前任务，不切换
    if used < TIME_SLICE {
        let _ = write!(serial, "yield_now: time slice not exhausted, continue current task\n");
        return; // 直接返回，继续执行当前任务
    }

    // 时间片耗尽，重置计数器，切换到调度器
    core.scheduler_used[current].store(0, Ordering::Release);

    let old_ctx = core.task_pool[current]
        .as_mut()
        .expect("current task should exist")
        .ctx
        .as_mut_ptr();
    let new_ctx = core.scheduler_ctx.as_mut_ptr();
    let _ = write!(serial, "yield_now: switching to scheduler\n");
    unsafe { switch_to(old_ctx, new_ctx); }
}

// 全局变量：当前运行所在的核心 ID（由每个核心自己设置）
static CURRENT_CORE_ID: AtomicUsize = AtomicUsize::new(0);

/// 每个 CPU 核心独立运行的“时间分配调度器”
/// 参数 core_id: 指定该调度器运行在哪一个逻辑核心上（由 AP 启动后调用）
extern "C" fn per_core_scheduler(core_id: usize) -> ! {
    CURRENT_CORE_ID.store(core_id, Ordering::Release); // 设置当前核心 ID
    let core = unsafe {
        &mut *CORE_DATA[core_id]
    };
    let mut serial = unsafe {
        SerialPort::new(0x3F8)
    };
    serial.init();
    let _ = write!(serial, "per_core_scheduler: task_count = {}\n", core.task_count.load(Ordering::Acquire));

    // 如果没有任务，则进入 halt 状态等待
    while core.task_count.load(Ordering::Acquire) == 0 {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack));
        }
    }

    let first_idx = (0..MAX_TASKS_PER_CORE)
        .find(|&i| core.task_pool[i].is_some())
        .expect("task pool should not be empty");
    core.current_task_id.store(first_idx, Ordering::Release);

    // 在首次切换前，初始化调度器上下文（占位）
    core.scheduler_ctx.write(TaskContext {
        rsp: 0,
    });

    let _ = write!(serial, "Before first switch\n");
    let new_ctx_ptr = core.task_pool[first_idx].as_ref().unwrap().ctx.as_ptr();
    let new_ctx_rsp = unsafe { (*new_ctx_ptr).rsp };
    let return_addr = unsafe { *( (new_ctx_rsp + 48) as *mut u64 ) }; // 调试：读返回地址
    let _ = write!(serial, "new_ctx.rsp={:#x}, return_addr={:#x}\n", new_ctx_rsp, return_addr);
    unsafe {
        switch_to(
            core.scheduler_ctx.as_mut_ptr(),
            core.task_pool[first_idx].as_ref().unwrap().ctx.as_ptr(),
        );
    }
    // 第一次 switch_to 后，控制权交给任务，任务 yield 后返回这里

    let _ = write!(serial, "Scheduler loop iteration\n");
    // ★ 任务 yield 回来之后，控制流从这里继续（永不退出）
    loop {
        let current = core.current_task_id.load(Ordering::Acquire);
        let n = core.task_count.load(Ordering::Acquire);
        let _ = write!(serial, "Loop: current={}, n={}\n", current, n);

        if n == 0 {
            unsafe {
                core::arch::asm!("hlt", options(nomem, nostack));
            }
            continue;
        }

        let mut next = current;
        let mut attempts = 0;
        // 循环查找下一个 Ready 任务（避开 Suspended 和 Terminated）
        while attempts < n {
            next = (next + 1) % n;
            if let Some(task) = &core.task_pool[next] {
                if TaskState::from_u8(task.state.load(Ordering::Acquire)) == TaskState::Ready {
                    break;
                }
            }
            attempts += 1;
        }
        if attempts == n { // 无就绪任务，进入 halt
            let _ = write!(serial, "Loop: no ready task\n");
            unsafe {
                core::arch::asm!("hlt", options(nomem, nostack));
            }
            continue;
        }

        let _ = write!(serial, "Loop: next={}\n", next);

        // 切换任务：从调度器上下文切换到目标任务
        let old_ctx = core.scheduler_ctx.as_mut_ptr(); // 调度器自身的上下文
        let new_ctx = core.task_pool[next].as_ref().unwrap().ctx.as_ptr();
        core.current_task_id.store(next, Ordering::Release);

        unsafe {
            switch_to(old_ctx, new_ctx);
        } // 切换后任务运行，直到 yield 或时间片耗尽
        // 任务 yield 后，控制流回到这里继续循环
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 子系统：多核APIC
////////////////////////////////////////////////////////////////////////////////////////////////////

// 多核架构基础定义
/// 最大支持的 CPU 核心数（民用电脑通常 4~16 核，256 是上限）
#[cfg(not(feature = "multicore"))] // 条件编译：未启用多核时，核心数为1（单核模式）
pub const MAX_CORES: usize = 1;

#[cfg(feature = "multicore")] // 启用多核时，支持最多256个核心（覆盖当前硬件上限）
pub const MAX_CORES: usize = 256;

static mut AP_IST_STACK: [u8; 4096 * 16] = [0; 4096 * 16]; // AP核心的中断栈（16页，共64KB），用于中断处理

#[cfg(feature = "multicore")] // 仅在多核模式下引用外部符号（来自汇编或链接脚本）
extern "C" {
    static ap_startup: u8; // AP启动代码的起始地址（由汇编提供）
    static bsp_cr3: u64; // BSP的CR3值（页表基址），供AP初始化时使用
    static ap_stack_top: u64; // AP临时栈的栈顶地址（汇编中定义）
    static rust_entry: u64; // AP进入Rust后跳转的函数地址（即ap_rust_main）
}

/// AP 核心（从核）被 BSP 唤醒后的 Rust 入口点
#[cfg(feature = "multicore")] // 只存在于多核编译中
#[no_mangle] // 保持符号名不变，供汇编调用
pub extern "C" fn ap_rust_main(apic_id: u32) -> ! { // 接收APIC ID作为参数，由汇编传入
    let core_id = apic_id as usize; // 将APIC ID转为数组索引
    if core_id >= MAX_CORES { // 越界保护：若ID超出支持范围则永久停机
        loop {
            unsafe {
                core::arch::asm!("hlt");
            }
        }
    }

    // 切换到该核心的独立栈（每个核心拥有自己的栈，避免竞争）
    static mut AP_STACKS: [[u8; STACK_SIZE]; MAX_CORES] = [[0; STACK_SIZE]; MAX_CORES]; // 所有核心的栈数组（静态分配）
    let stack_top = unsafe {
        AP_STACKS[core_id].as_ptr() as u64 + STACK_SIZE as u64
    }; // 计算该核心栈的栈顶（高地址）
    unsafe {
        core::arch::asm!("mov rsp, {}", in(reg) stack_top);
    } // 将RSP切换到该核心的专用栈

    let core = unsafe {
        &mut *CORE_DATA[core_id]
    }; // 获取该核心的PerCoreData可变引用（先前已初始化）

    unsafe {
        // 1. 使用新版函数初始化 GDT/TSS/IST（为每个核心独立设置）
        let ist_top = VirtAddr::from_ptr(AP_IST_STACK.as_mut_ptr().add(AP_IST_STACK.len())); // IST栈顶（用于中断栈）
        init_gdt_tss_ist(&mut core.gdt, &mut core.tss, ist_top); // 初始化该核心的GDT、TSS和IST栈

        // 2. 配置 AP 核心独立的 IDT（必须与 BSP 独立，因为每个核心有独立的中断处理）
        core.idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(0); // 设置双重故障处理，使用IST栈索引0
        core.idt.general_protection_fault.set_handler_fn(gp_handler); // 通用保护故障
        core.idt.page_fault.set_handler_fn(page_fault_handler); // 页故障
        core.idt[0x20].set_handler_fn(timer_handler); // 定时器中断（IRQ0，向量0x20）
        // 可选的兜底异常可根据需要补充（如断点、除零等）
        core.idt.load(); // 加载该核心的IDT（使用lidt指令）

        unsafe {
            init_apic_timer();
        } // 初始化该核心的APIC定时器
        // 3. 开启中断
        use x86_64::instructions::interrupts;
        interrupts::enable(); // 启用该核心的中断响应

        // 4. 进入该核心独立的调度循环（永不返回）
        per_core_scheduler(core_id);
    }
}

/// BSP 主核唤醒所有 AP 从核的函数
#[cfg(feature = "multicore")] // 仅在多核模式下编译
pub unsafe fn wake_up_aps() {
    const AP_BASE: u64 = 0x8000; // AP启动代码被复制到的物理地址（低内存区域，实模式可用）
    const BSP_CR3_OFFSET: u64 = 0x28; // 在启动代码二进制中的偏移，用于写入BSP的CR3值
    const AP_STACK_TOP_OFFSET: u64 = 0x30; // 偏移用于写入临时栈顶
    const RUST_ENTRY_OFFSET: u64 = 0x38; // 偏移用于写入Rust入口函数地址

    let ap_dest_virt = HIGH_BASE + AP_BASE; // 计算AP_BASE对应的虚拟地址（通过HIGH_BASE映射）
    core::ptr::copy_nonoverlapping( // 将AP_STARTUP_BIN（编译时嵌入的AP启动代码）复制到目标物理地址（不重叠）
                                    AP_STARTUP_BIN.as_ptr(),
                                    ap_dest_virt as *mut u8,
                                    4096 // 复制一页（4KB）足够容纳启动代码
    );

    let (cr3_frame, _) = Cr3::read(); // 读取当前BSP的CR3（页表物理地址）
    let bsp_cr3_ptr = (AP_BASE + BSP_CR3_OFFSET) as *mut u64; // 计算写入CR3的物理地址
    *bsp_cr3_ptr = cr3_frame.start_address().as_u64(); // 写入BSP的CR3值，供AP启动时加载

    const TEMP_STACK_TOP: u64 = 0x9000 + 4096; // AP临时栈的栈顶地址（0x9000页顶）
    let ap_stack_top_ptr = (AP_BASE + AP_STACK_TOP_OFFSET) as *mut u64; // 写入栈顶的地址
    *ap_stack_top_ptr = TEMP_STACK_TOP; // 设置临时栈顶

    let rust_entry_ptr = (AP_BASE + RUST_ENTRY_OFFSET) as *mut u64; // 写入Rust入口函数地址
    *rust_entry_ptr = ap_rust_main as u64; // 将ap_rust_main的函数指针写入

    // 发送 INIT/SIPI IPI（处理器间中断）唤醒AP
    let apic_base = 0xFEE00000u64; // APIC MMIO基地址（固定）
    let icr_low = apic_base + 0x300; // 中断命令寄存器低32位地址
    core::ptr::write_volatile(icr_low as *mut u32, 0x000C4500); // 发送INIT IPI（中断类型INIT，清除AP状态）
    for _ in 0..10000 {
        core::arch::asm!("pause");
    } // 延迟等待（约10ms）
    for _ in 0..2 { // 发送两次SIPI（启动IPI），确保AP收到
        core::ptr::write_volatile(icr_low as *mut u32, 0x000C4608); // SIPI，向量8（即AP启动代码的物理地址0x8000 >> 12）
        for _ in 0..200 {
            core::arch::asm!("pause");
        } // 短延迟
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统5：安全入口桩（GDT、TSS、IST 栈）
////////////////////////////////////////////////////////////////////////////////////////////////////

// 1. 静态分配 GDT、TSS 和 IST 栈
static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new(); // 全局描述符表（每个核心独立，此处为BSP初始用，实际应参数化）
static mut TSS: TaskStateSegment = TaskStateSegment::new(); // 任务状态段（用于特权级切换和中断栈）
// #DF 需要 16KB 的独立安全栈（双重故障必须使用独立栈，避免递归崩溃）
const IST_STACK_SIZE: usize = 4096 * 16; // 16KB，足够容纳异常处理帧
static mut IST_STACK: [u8; IST_STACK_SIZE] = [0; IST_STACK_SIZE]; // 静态分配中断栈，确保在内存中固定

// 修改现有的 init_gdt_tss_ist 为参数化版本（支持多核独立GDT/TSS）
pub unsafe fn init_gdt_tss_ist(
    gdt: &'static mut GlobalDescriptorTable, // 传入对应核心的GDT（'static确保生命周期与内核相同）
    tss: &'static mut TaskStateSegment, // 对应核心的TSS
    ist_stack_top: VirtAddr, // 该核心IST栈的虚拟地址（栈顶，因为x86栈向下生长，但这里填入的是栈顶地址，TSS期望是栈顶）
) {
    // 将该核心的 IST 栈顶地址填入 TSS 的 0 号槽位（索引0对应#DF使用）
    tss.interrupt_stack_table[0] = ist_stack_top;
    // 将 TSS 描述符添加到 GDT 并获取选择子（TSS是系统段）
    let tss_selector = gdt.append(Descriptor::tss_segment(tss));
    // 加载 GDT 和 TR 寄存器（使新GDT生效，并加载TSS选择子到TR）
    gdt.load();
    load_tss(tss_selector);
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new(); // 中断描述符表（每个核心独立，由各自IDT管理）

extern "x86-interrupt" fn page_fault_handler(
    _stack: InterruptStackFrame,
    _code: PageFaultErrorCode,
) {
    // 1. 初始化串口（因中断可能发生在任何时刻，需要独立初始化串口）
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();

    // 2. 读取 CR2 寄存器，获取导致缺页的虚拟地址（使用 unwrap 取出 Result 里的地址）
    let fault_addr = x86_64::registers::control::Cr2::read().unwrap(); // CR2在缺页时自动保存出错地址

    // 3. 使用 .bits() 获取底层 u64 值，进行位运算
    let err = _code.bits();
    let present = (err & 1) != 0;          // Bit 0: 页面是否存在（0表示缺页，1表示保护违规）
    let write = (err & (1 << 1)) != 0;     // Bit 1: 是否为写操作导致（0读，1写）
    let user = (err & (1 << 2)) != 0;      // Bit 2: 是否由用户态访问导致（0内核，1用户）

    // 4. 打印错误信息
    write!(
        serial,
        ">>> #PF ERROR: Accessing virtual address {:#x}. Present: {}, Write: {}, User: {}\n",
        fault_addr,
        present,
        write,
        user
    ).unwrap();

    // 5. 停机死循环（内核无法恢复页错误，直接停机）
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

extern "x86-interrupt" fn gp_handler(
    _stack: InterruptStackFrame,
    _code: u64,
) {
    // 1. 在中断上下文里重新初始化串口
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();

    // 2. 打印错误信息
    // `_stack.instruction_pointer` 是触发异常的指令地址
    // `_code` 包含出错的 GDT 选择子索引（低16位）和外部错误码
    unsafe {
        write!(
            serial,
            ">>> #GP ERROR: Error Code {:#x}, RIP {:#x}\n",
            _code,
            _stack.instruction_pointer.as_u64()
        ).unwrap();
        write!(serial, "Kernel Panic: GDT or Segment error. Halting.\n").unwrap();
    }

    // 3. 停机死循环
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

extern "x86-interrupt" fn double_fault_handler(
    _stack: InterruptStackFrame,
    _code: u64,
) -> ! {
    // 双重故障说明发生了嵌套异常，属于致命硬件错误，直接停机
    let mut serial = unsafe {
        SerialPort::new(0x3F8)
    };
    serial.init();
    unsafe {
        write!(
            serial,
            ">>> #DF ERROR: Recursive fault at RIP {:#x}, Error Code {:#x}\n",
            _stack.instruction_pointer.as_u64(),
            _code
        ).unwrap();
        write!(serial, "FATAL DOUBLE FAULT! System Halted.\n").unwrap();
    }

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// 不带错误码的兜底异常（用于 #DE, #BP, #UD, #NM 等），使用统一处理函数
extern "x86-interrupt" fn unhandled_exception_handler(stack: InterruptStackFrame) {
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();
    let _ = write!(serial, "[PANIC] Unhandled Exception! RIP: {:#x}\n", stack.instruction_pointer.as_u64());
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// 带错误码的兜底异常（用于 #TS, #NP, #SS, #AC, #CP 等），携带额外错误码
extern "x86-interrupt" fn unhandled_exception_handler_with_code(stack: InterruptStackFrame, code: u64) {
    let mut serial = unsafe {
        SerialPort::new(0x3F8)
    };
    serial.init();
    let _ = write!(serial, "[PANIC] Unhandled Exception (Error Code: {:#x})! RIP: {:#x}\n", code, stack.instruction_pointer.as_u64());
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

const APIC_BASE: u64 = 0xFEE0_0000; // 本地APIC MMIO基地址（固定，物理地址）

pub unsafe fn init_idt() {
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: start\n");

    // --- 必须处理的三个核心异常 ---
    IDT.double_fault.set_handler_fn(double_fault_handler).set_stack_index(0); // #DF使用IST栈0
    IDT.general_protection_fault.set_handler_fn(gp_handler); // #GP（通用保护）
    IDT.page_fault.set_handler_fn(page_fault_handler); // #PF（页错误）
    IDT[0x20].set_handler_fn(timer_handler); // 定时器中断向量（IRQ0映射到0x20）
    IDT[0x21].set_handler_fn(unhandled_exception_handler); // 其他硬件中断暂时不处理（例如键盘）
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: core handlers set\n");

    // --- 不带错误码的异常 ---
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: before divide_error\n");
    IDT.divide_error.set_handler_fn(unhandled_exception_handler); // #DE
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: after divide_error\n");
    IDT.debug.set_handler_fn(unhandled_exception_handler); // #DB
    IDT.breakpoint.set_handler_fn(unhandled_exception_handler); // #BP
    IDT.overflow.set_handler_fn(unhandled_exception_handler); // #OF
    IDT.bound_range_exceeded.set_handler_fn(unhandled_exception_handler); // #BR
    IDT.invalid_opcode.set_handler_fn(unhandled_exception_handler); // #UD
    IDT.device_not_available.set_handler_fn(unhandled_exception_handler); // #NM
    IDT.x87_floating_point.set_handler_fn(unhandled_exception_handler); // #MF
    IDT.simd_floating_point.set_handler_fn(unhandled_exception_handler); // #XM
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: no-err handlers set\n");

    // --- 带错误码的异常（全部用带 code 的兜底函数） ---
    IDT.invalid_tss.set_handler_fn(unhandled_exception_handler_with_code); // #TS
    IDT.segment_not_present.set_handler_fn(unhandled_exception_handler_with_code); // #NP
    IDT.stack_segment_fault.set_handler_fn(unhandled_exception_handler_with_code); // #SS
    IDT.alignment_check.set_handler_fn(unhandled_exception_handler_with_code); // #AC
    IDT.machine_check.set_handler_fn(machine_check_handler); // #MC（机器检查，特殊处理）
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: err-code handlers set\n");

    let _ = write!(DEBUG_SERIAL.lock(), "IDT: before load\n");
    IDT.load(); // 加载IDT到CPU（lidt指令）
    let _ = write!(DEBUG_SERIAL.lock(), "IDT: after load\n");
}

pub unsafe fn init_apic_timer() {
    // 0x100 | 0x21 = 启用APIC（第8位），并将虚假中断向量号设为 0x21 (33)，用于处理未预期的中断
    core::ptr::write_volatile((APIC_BASE + 0xF0) as *mut u32, 0x100 | 0x21); // 写入Spurious Interrupt Vector Register
    core::ptr::write_volatile((APIC_BASE + 0x320) as *mut u32, 0x20000 | 0x20); // 设置LVT Timer寄存器：周期性模式（bit17），向量0x20
    core::ptr::write_volatile((APIC_BASE + 0x3E0) as *mut u32, 0x3); // 设置Divide Configuration Register（分频配置），0x3表示分频因子16
    core::ptr::write_volatile((APIC_BASE + 0x380) as *mut u32, 0x10_0000); // 设置初始计数寄存器（0x10_0000），以产生约100Hz的中断（基于总线频率）
}

// 专门处理 #MC 机器检查异常（致命硬件错误，永不返回）
extern "x86-interrupt" fn machine_check_handler(_stack: InterruptStackFrame) -> ! {
    let mut serial = unsafe { SerialPort::new(0x3F8) };
    serial.init();
    let _ = write!(serial, "[PANIC] Machine Check Exception (MC)! Hardware fatal error.\n");
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

extern "x86-interrupt" fn timer_handler(_stack: InterruptStackFrame) {
    unsafe {
        core::ptr::write_volatile((APIC_BASE + 0xB0) as *mut u32, 0);
    } // 向EOI寄存器写入0，通知APIC中断处理完成

    let core_id = CURRENT_CORE_ID.load(Ordering::Acquire); // 获取当前CPU核心ID
    let core = unsafe {
        &mut *CORE_DATA[core_id]
    }; // 获取该核心的PerCoreData
    let current = core.current_task_id.load(Ordering::Acquire); // 当前正在运行的任务槽位

    // 递增当前任务的已用时间片
    let used = core.scheduler_used[current].load(Ordering::Acquire) + 1; // 增加1 tick
    core.scheduler_used[current].store(used, Ordering::Release); // 写回
    // 也可以递增全局 tick（用于全局时钟）
    core.scheduler_t.fetch_add(1, Ordering::Release); // 全局tick计数加1
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 系统整合：支持用户态(Ring3)
////////////////////////////////////////////////////////////////////////////////////////////////////

// 1. 黑匣子（诊断缓冲区）
const DIAG_BUFFER_SIZE: usize = 512; // 诊断缓冲区容量，记录最近的512次故障签名
#[repr(C)] // 确保结构体按C语言布局，便于串口输出或外部解析
struct FaultSignature {
    context_id: u64, // 发生故障的执行上下文ID
    faulting_token_id: u64, // 导致故障的令牌ID
    call_chain_hash: u64, // 当前调用链的哈希值，用于溯源委托链
    tick: u64, // 故障发生时的系统tick计数
}
static DIAG_BUFFER: Mutex<[MaybeUninit<FaultSignature>; DIAG_BUFFER_SIZE]> = Mutex::new([const { MaybeUninit::uninit() }; DIAG_BUFFER_SIZE]); // 静态诊断缓冲区，用锁保护并发写入，MaybeUninit避免初始化开销
static DIAG_HEAD: AtomicUsize = AtomicUsize::new(0); // 环形缓冲区的头指针（原子操作），记录下一个写入位置

pub fn record_fault(ctx_id: u64, token_id: u64, chain_hash: u64) {
    let mut buffer = DIAG_BUFFER.lock(); // 获取互斥锁，防止多核同时写入
    let head = DIAG_HEAD.load(Ordering::Acquire); // 原子加载当前头指针（Acquire确保后续写入可见）
    buffer[head % DIAG_BUFFER_SIZE].write(FaultSignature { context_id: ctx_id, faulting_token_id: token_id, call_chain_hash: chain_hash, tick: 0 }); // 在环形缓冲区对应位置写入故障记录（tick暂为0，实际应传入）
    DIAG_HEAD.store(head + 1, Ordering::Release); // 更新头指针（Release确保之前的写入对其他CPU可见）
}

//  2. MPK 特权分级分配器
static MPK_ALLOC: AtomicU8 = AtomicU8::new(0); // 位图原子变量，每个bit表示对应MPK密钥是否已被分配（0空闲，1占用）
pub fn alloc_mpk_key() -> Option<u8> {
    for key in 1..8 { // MPK密钥1~7可用（0通常保留给内核）
        if MPK_ALLOC.fetch_or(1 << key, Ordering::AcqRel) & (1 << key) == 0 { // 原子地将对应位置1，并检查旧值中的该位是否为0（若为0则说明我们成功抢占了该密钥）
            return Some(key); // 成功分配，返回密钥编号
        }
    }
    None // 所有密钥均被占用
}
pub unsafe fn switch_mpk_context(new_mpk_key: u8) {
    let mut pkrs = 0u64; // PKRU寄存器值，控制每个密钥的读/写权限（每4位控制一个密钥）
    for i in 0..16 { // 遍历所有16个密钥（0~15）
        pkrs |= if i == new_mpk_key { 0 } else { 0xFFFF } << (i * 4); // 只有目标密钥给予全权限（0表示允许读写），其他密钥全部禁止（0xFFFF表示禁止读和写）
    }
    Msr::new(0x6E1).write(pkrs); // 写入PKRU模型特定寄存器（MSR 0x6E1），立即生效
}

/// 用户态操作类型（符合宪法“存储程序与数据同权”）
#[derive(Debug, Clone, Copy)]
pub enum TokenAction {
    Read,      // 读权限 (0b001)
    Write,     // 写权限 (0b010)
    Execute,   // 执行权限 (0b100)
}

/// 用户态令牌守卫（RAII 自动释放）
/// 当用户态持有此守卫时，表示已成功获取访问权限。
pub struct TokenGuard<'a> {
    token_id: u64,
    owner_id: u64,
    // 守卫持有一个指向内核管理器的不变引用，用于 Drop 时释放
    manager: &'a TokenManager,
    // 线性类型标记，防止被意外复制
    _marker: PhantomData<*mut ()>,
}

pub unsafe fn enter_user_mode(rip: u64, rsp: u64, _pkru: u32) -> ! {
    asm!(
    "push {ss}",
    "push {rsp}",
    "pushfq",
    "push {cs}",
    "push {rip}",
    "iretq",
    ss = const(0x2B),
    cs = const(0x33),
    rip = in(reg) rip,
    rsp = in(reg) rsp,
    options(noreturn)
    );
    // 永远不会到这里，但为了类型一致
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

//impl Drop for TokenGuard<'_> {
    //fn drop(&mut self) {
    //    // 减少引用计数
    //    let new_count = self.obj.ref_count.fetch_sub(1, Ordering::Release) - 1;
     //   if new_count == 0 {
      //      // 释放对象
      //      self.manager.release_object(self.object_index);
      //  }
    //}
//}

/// 能力门：用户态访问内核资源的唯一入口
/// 这是一个直接函数调用，而不是系统调用。
pub fn use_token<'a>(
    token_id: u64,
    owner_id: u64,
    action: TokenAction,
    buf: &'a mut [u8],
) -> Result<&'a mut [u8], &'static str> {
    // 1. 将用户态操作类型转换为内核权限位掩码
    let perms = match action {
        TokenAction::Read => 0b001,
        TokenAction::Write => 0b010,
        TokenAction::Execute => 0b100,
    };

    // 2. 调用内核层验证（纯函数调用，无特权级切换）
    // 如果验证通过，则返回可变引用；否则返回错误。
    // 注意：这里不涉及 MPK、IOMMU，只涉及身份验证和令牌有效性。
    if TOKEN_MANAGER.lock().try_acquire(token_id, perms, owner_id) { // 获取全局锁并验证令牌
        // 验证通过，返回原地址 buf 的可变引用（零拷贝，因为用户态与内核共享同一地址空间）
        Ok(buf)
    } else {
        // 验证失败，返回错误（内核会记录到黑匣子，但黑匣子属于内核层）
        Err("Access denied by capability engine")
    }
}

/// 获取当前任务的初始能力令牌 ID（由调度器保存）
#[no_mangle]
pub extern "C" fn get_current_token() -> u64 {
    let core_id = CURRENT_CORE_ID.load(Ordering::Acquire);
    let core = unsafe { &mut *CORE_DATA[core_id] };
    let current = core.current_task_id.load(Ordering::Acquire);
    if let Some(task) = &core.task_pool[current] {
        task.init_token
    } else {
        0
    }
}

/// 能力门原始接口（供 C ABI 调用）
/// 参数：token_id, owner_id, action(0b001读,0b010写,0b100执行), buf_ptr, len
/// 返回：true 表示成功，false 表示失败

/// 获取当前任务的初始能力令牌 ID（由调度器保存）

#[no_mangle]
pub extern "C" fn use_token_raw(
    token_id: u64,
    owner_id: u64,
    action: u8,
    buf_ptr: *mut u8,
    len: usize,
) -> bool {
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr, len) };
    let action = match action {
        0b001 => TokenAction::Read,
        0b010 => TokenAction::Write,
        0b100 => TokenAction::Execute,
        _ => return false,
    };
    use_token(token_id, owner_id, action, buf).is_ok()
}

/// 用户态文件令牌（高级包装）
pub struct FileToken {
    token_id: u64,
    owner_id: u64,
    path: Option<&'static str>, // 可选的路径信息，用于高级查找（实际不影响权限）
}

impl FileToken {
    pub fn new(token_id: u64, owner_id: u64) -> Self {
        Self { token_id, owner_id, path: None }
    }

    pub fn read<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], &'static str> {
        use_token(self.token_id, self.owner_id, TokenAction::Read, buf) // 直接调用能力门
    }
}

/// 用户态通道令牌（发布-订阅通信）
pub struct ChannelToken {
    token_id: u64,
    owner_id: u64,
}

impl ChannelToken {
    pub fn new(token_id: u64, owner_id: u64) -> Self {
        Self { token_id, owner_id }
    }

    pub fn read<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut [u8], &'static str> {
        use_token(self.token_id, self.owner_id, TokenAction::Read, buf)
    }
}

#[derive(Copy, Clone)] // 允许复制，但应谨慎使用（实际上DMA映射不可复制，此处仅为示例）
struct DmaMapping {
    token_id: u64,
    phys_base: u64, // DMA映射的物理地址起始
    len: usize, // 映射长度
}

// 5. IOMMU 与 DMA 直通
static DMA_MAPPINGS: Mutex<[Option<DmaMapping>; 16]> = Mutex::new([None; 16]); // 静态DMA映射表，最多16个条目（实际应根据硬件调整）

pub unsafe fn configure_iommu(token: &TokenEntry, phys_base: u64, len: usize) -> bool {
    if token.permissions & 0b1000 == 0 { return false; } // 检查令牌是否具有IOMMU_CFG权限（位3）
    let mut mappings = DMA_MAPPINGS.lock(); // 获取映射表锁
    for slot in mappings.iter_mut() {
        if slot.is_none() { // 找到空槽位
            *slot = Some(DmaMapping { token_id: token.id, phys_base, len }); // 记录映射
            return true;
        }
    }
    false // 没有空槽位，配置失败
}

pub unsafe fn dma_unmap(token_id: u64) {
    let mut mappings = DMA_MAPPINGS.lock(); // 加锁
    for slot in mappings.iter_mut() {
        if let Some(m) = slot {
            if m.token_id == token_id {
                *slot = None; // 清空该槽位，释放映射（实际需清理IOMMU页表，此处仅为占位）
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
/// 函数入口
////////////////////////////////////////////////////////////////////////////////////////////////////

#[entry] // UEFI 应用入口点宏，告诉编译器此函数为 UEFI 引导入口
fn main(_image_handle: uefi::Handle, system_table: SystemTable<Boot>) -> Status { // UEFI 主函数，返回 UEFI 状态码
    uefi::helpers::init().unwrap(); // 初始化 UEFI 帮助库（如全局分配器、日志等），失败则 panic

    let _ = write!(DEBUG_SERIAL.lock(), "----------------------------------------------------\n");
    let _ = write!(DEBUG_SERIAL.lock(), "新型内核思想：彻底统一编址，语言既是隔离墙，能力授权引擎\n");
    let _ = write!(DEBUG_SERIAL.lock(), "----------------------------------------------------\n");
    // 1. 初始化堆
    unsafe {
        ALLOCATOR.lock().init(HEAP.as_mut_ptr(), HEAP_SIZE);
    } // 全局分配器锁定并初始化，使用静态堆内存，必须 unsafe 因为涉及裸指针
    status_ok!("初始化堆"); // 宏：输出绿色成功信息到串口

    // 2. 初始化全局调试串口
    DEBUG_SERIAL.lock().init(); // 获取串口互斥锁并初始化硬件（设置波特率等）
    status_ok!("初始化串口");

    // 3. 获取内存映射，初始化物理内存位图
    let boot_services = system_table.boot_services();
    let memory_map_result = boot_services.memory_map(MemoryType::LOADER_DATA);

    match memory_map_result {
        Ok(map) => {
            // === 使用 UEFI 内存映射初始化位图 ===
            unsafe {
                // 1. 先将所有位设为 1（全部占用）
                for i in 0..BITMAP_LEN {
                    BITMAP[i].store(u64::MAX, Ordering::Release);
                }

                // 2. 遍历 UEFI 内存描述符，将 CONVENTIONAL 区域标记为空闲（清 0）
                for desc in map.entries() {
                    if desc.ty == MemoryType::CONVENTIONAL {
                        let start_frame = desc.phys_start / 4096;
                        let page_count = desc.page_count;
                        for offset in 0..page_count {
                            let current_frame = start_frame + offset;
                            if current_frame >= MAX_PHYS_MEM_PAGES as u64 {
                                break;
                            }
                            let idx = (current_frame / 64) as usize;
                            let bit = (current_frame % 64) as u32;
                            BITMAP[idx].fetch_and(!(1 << bit), Ordering::AcqRel);
                        }
                    }
                }
            }
        }
        Err(e) => {
            let _ = write!(DEBUG_SERIAL.lock(), "Memory map error: {:?}, using fallback bitmap\n", e);
            // === 后备位图（保守策略）：仅暴露 1MB ~ 2MB 区域 ===
            unsafe {
                // 1. 全部置 1（占用）
                for i in 0..BITMAP_LEN {
                    BITMAP[i].store(u64::MAX, Ordering::Release);
                }

                // 2. 只把 0x100000 ~ 0x200000（1MB ~ 2MB）标记为空闲
                let start_frame = 0x100000 / 4096;
                let page_count = 0x100000 / 4096;
                for offset in 0..page_count {
                    let current_frame = start_frame + offset;
                    if current_frame >= MAX_PHYS_MEM_PAGES as u64 {
                        break;
                    }
                    let idx = (current_frame / 64) as usize;
                    let bit = (current_frame % 64) as u32;
                    BITMAP[idx].fetch_and(!(1 << bit), Ordering::AcqRel);
                }
            }
        }
    }
    status_ok!("建立物理内存位图");

    // 5. 测试分配
    unsafe {
        if let Some(phys) = alloc_page() { // 尝试分配一页物理内存（空闲页）
            let _ = write!(DEBUG_SERIAL.lock(), "Test Alloc Success: {:#x}\n", phys);
        }
    }
    status_ok!("物理内存分配测试");

    // 5.1. 在启用 APIC 之前冻结页表，切换到 HIGH_BASE
    unsafe {
        if let Err(e) = init_paging() { // 构建非恒等映射（HIGH_BASE + phys），切换到新页表
            let _ = write!(DEBUG_SERIAL.lock(), "init_paging failed: {}\n", e);
            panic!("FATAL: Page table freeze failed"); // 页表切换失败则直接停机
        }
    }
    status_ok!("页表已冻结，切换到 HIGH_BASE 非恒等映射");

    unsafe {
        const MAX_ACTIVE_CORES: usize = 4;  // 可改为 MAX_CORES（当前固定为4，简化演示）
        for core_id in 0..MAX_CORES { // 初始化每个核心的 PerCoreData（BSP+AP）
            // scheduler_used 数组：记录每个任务已用的 tick 数，初始全0
            let mut scheduler_used: [AtomicUsize; MAX_TASKS_PER_CORE] = core::mem::zeroed(); // 零初始化（全0）
            for elem in &mut scheduler_used {
                *elem = AtomicUsize::new(0); // 每个元素设置为原子类型初始0
            }

            // task_pool 初始化：每个槽位设为 None（无任务）
            let mut task_pool_uninit: [MaybeUninit<Option<Task>>; MAX_TASKS_PER_CORE] =
                core::mem::zeroed(); // 未初始化数组（零值）
            for slot in &mut task_pool_uninit {
                slot.write(None); // 写入 None
            }
            let task_pool: [Option<Task>; MAX_TASKS_PER_CORE] =
                core::mem::transmute(task_pool_uninit); // 将 MaybeUninit 数组转换为普通数组（安全，因已全部写入）

            let per_core = PerCoreData { // 构造每个核心的数据
                core_id,
                gdt: GlobalDescriptorTable::new(), // 空 GDT，稍后由 init_gdt_tss_ist 填充
                tss: TaskStateSegment::new(), // 空 TSS，稍后设置 IST
                idt: InterruptDescriptorTable::new(), // 空 IDT，稍后设置处理器
                current_task_id: AtomicUsize::new(0),
                scheduler_t: AtomicUsize::new(0), // 全局 tick 计数
                scheduler_used,
                task_pool,
                task_count: AtomicUsize::new(0), // 初始无任务
                scheduler_ctx: MaybeUninit::uninit(), // 调度器上下文未初始化
            };
            // 计算 PerCoreData 结构体需要多少物理页
            let size = core::mem::size_of::<PerCoreData>();
            let num_pages = (size + 4095) / 4096; // 向上取整

            // 分配连续物理页
            let phys_start = alloc_pages(num_pages).expect("Failed to allocate PerCoreData");

            // 转换为虚拟地址（加上 HIGH_BASE）
            let virt_start = (phys_start + HIGH_BASE) as *mut PerCoreData;

            // 将 per_core 数据写入动态分配的内存
            virt_start.write(per_core);

            // 存储指针到全局数组
            CORE_DATA[core_id] = virt_start;
        }

        // BSP 配置（核心0）
        let bsp_core = unsafe { &mut *CORE_DATA[0] }; // 获取 BSP 核心的可变引用
        let ist_top = VirtAddr::from_ptr(IST_STACK.as_ptr().add(IST_STACK.len())); // IST 栈顶地址（高地址）
        init_gdt_tss_ist(&mut bsp_core.gdt, &mut bsp_core.tss, ist_top); // 初始化 BSP 的 GDT/TSS/IST
        status_ok!("BSP GDT/TSS 初始化");

        // 为 BSP 设置 IDT 处理程序
        bsp_core.idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(0); // 双重故障使用 IST 栈0
        bsp_core.idt.general_protection_fault.set_handler_fn(gp_handler);
        bsp_core.idt.page_fault.set_handler_fn(page_fault_handler);
        bsp_core.idt[0x20].set_handler_fn(timer_handler); // 定时器中断向量 0x20（IRQ0）
        bsp_core.idt[0x21].set_handler_fn(unhandled_exception_handler); // 其他中断（如键盘）暂不处理
        bsp_core.idt.load(); // 加载 IDT 到 CPU（lidt 指令）
        status_ok!("IDT 初始化");
    }

    // 7. 必须先初始化 APIC 定时器和任务池
    unsafe { init_apic_timer(); } // 初始化本地 APIC 定时器，使其产生周期中断
    status_ok!("APIC 定时器初始化");

    let _ = write!(DEBUG_SERIAL.lock(), "注意：根密钥在于键值存储开发之后内核描定\n");
    unsafe {
        GLOBAL_SALT = 0xDEADBEEF_CAFEBABE;
    }
    let _ = write!(DEBUG_SERIAL.lock(), "----------------------------------------------------\n");
    let _ = write!(DEBUG_SERIAL.lock(), "新型操作系统开发团队-为了民用而生\n");
    let _ = write!(DEBUG_SERIAL.lock(), "----------------------------------------------------\n");

    status_ok!("准备添加任务");
    unsafe {
        loop {
            let token = add_task(0, task_a_wrapper); // 在核心0上添加任务 A，返回初始令牌 ID
            if token.is_some() {
                status_ok!("添加任务 A 成功");
            } else {
                status_fail!("添加任务 A 失败");
                break;  // 任务池可能已满，退出循环
            }

            let token = add_task(0, task_b_wrapper); // 添加任务 B
            if token.is_some() {
                status_ok!("添加任务 B 成功");
            } else {
                status_fail!("添加任务 B 失败");
                break;
            }
        }
    }

    // 8. 然后唤醒所有 AP 核心（多核启动）
    #[cfg(feature = "multicore")] // 仅当启用多核特性时编译
    unsafe {
        wake_up_aps();
    } // BSP 发送 IPI 唤醒所有 AP，使它们进入各自调度循环

    // 9. 最后启动 BSP 核心的调度器（永不返回）
    status_ok!("启动调度器");
    per_core_scheduler(0); // 当前核心（BSP）进入调度循环，从此不再返回
}

#[panic_handler] // 标记该函数为 panic 处理函数，当 Rust 发生 panic 时调用
fn panic(info: &PanicInfo) -> ! { // 接收 PanicInfo 结构体引用，返回类型为 ! 表示永不返回（发散函数）
    let mut serial = unsafe {
        SerialPort::new(0x3F8)
    }; // 创建串口实例，使用 COM1 端口（物理地址 0x3F8），unsafe 是因为裸机操作
    serial.init(); // 初始化串口硬件（设置波特率 115200、8N1 等）
    // 增加换行符和刷新（将 panic 信息格式化为字符串并写入串口）
    let _ = write!(serial, "KERNEL PANIC: {}\n", info); // 调用 write! 宏，忽略返回的 Result（失败也无法处理）
    // 强制刷新串口缓冲区（可选），此处插入一个 nop 指令作为轻量级刷新延迟
    unsafe {
        core::arch::asm!("nop");
    }
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    } // 进入死循环，执行 hlt 指令使 CPU 停机，等待外部中断唤醒（但此处永不返回）
}