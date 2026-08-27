org 0x8000
BITS 16

ap_startup:
    cli
    cld
    lgdt [gdt_48]
    mov eax, cr0
    or  al, 1
    mov cr0, eax
    push dword 0x08
    push dword ap_protected
    retf

BITS 32
ap_protected:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov eax, cr4
    or  eax, (1 << 5)
    mov cr4, eax
    mov ecx, 0xC0000080
    rdmsr
    or  eax, (1 << 8)
    wrmsr
    mov eax, [bsp_cr3]
    mov cr3, eax
    mov eax, cr0
    or  eax, 0x80000000
    mov cr0, eax
    push dword 0x08
    push dword ap_longmode
    retf

BITS 64
ap_longmode:
    ; 读取 APIC ID -> EDI
    mov eax, 0xFEE00020
    mov eax, [eax]
    shr eax, 24
    mov edi, eax

    mov rsp, [ap_stack_top]
    jmp [rust_entry]

align 8
gdt:
    dq 0x0000000000000000
    dq 0x00AF9A000000FFFF
    dq 0x00CF92000000FFFF
gdt_48:
    dw (gdt_48 - gdt - 1)
    dq gdt

align 8
bsp_cr3:
    dq 0
ap_stack_top:
    dq 0
rust_entry:
    dq 0

times 4096 - ($ - ap_startup) db 0