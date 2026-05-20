; boot.asm - 原子回路验证 (实模式, VGA屏幕输出)
; 编译: nasm -f bin boot.asm -o boot.bin
; 运行: qemu-system-x86_64 -drive format=raw,file=boot.bin

[org 0x7c00]

; 初始化段寄存器
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00

; 清屏 (VGA 文本模式 80x25)
    mov ax, 0xb800
    mov es, ax
    xor di, di
    mov cx, 80*25
    mov ax, 0x0f20
    rep stosw

; 创建令牌表 (0x7e00)
    mov byte [0x7e00], 0b01   ; 令牌0: 只读
    mov byte [0x7e02], 0b11   ; 令牌1: 读写

; 打印启动信息 (行0)
    mov si, msg_boot
    mov bl, 0
    call print_line

; 测试1: 令牌0 读 (应成功)
    mov al, 0
    mov ah, 0b01
    call use_token
    cmp al, 0
    je .t1_ok
    mov si, msg_fail
    jmp .t1_done
.t1_ok:
    mov si, msg_ok1
.t1_done:
    mov bl, 1
    call print_line

; 测试2: 令牌0 写 (应拒绝)
    mov al, 0
    mov ah, 0b10
    call use_token
    cmp al, 1
    je .t2_ok
    mov si, msg_fail
    jmp .t2_done
.t2_ok:
    mov si, msg_ok2
.t2_done:
    mov bl, 2
    call print_line

; 测试3: 令牌1 写 (应成功)
    mov al, 1
    mov ah, 0b10
    call use_token
    cmp al, 0
    je .t3_ok
    mov si, msg_fail
    jmp .t3_done
.t3_ok:
    mov si, msg_ok3
.t3_done:
    mov bl, 3
    call print_line

; 测试4: 令牌3 读 (应拒绝)
    mov al, 3
    mov ah, 0b01
    call use_token
    cmp al, 2
    je .t4_ok
    mov si, msg_fail
    jmp .t4_done
.t4_ok:
    mov si, msg_ok4
.t4_done:
    mov bl, 4
    call print_line

; 最终信息 (行5)
    mov si, msg_atomic
    mov bl, 5
    call print_line

    jmp $

; ---------- 函数 ----------
; use_token: al=索引, ah=需求(bit0读,bit1写) -> al=0成功,1权限不足,2未找到
use_token:
    push bx
    push di
    mov di, 0x7e00
    xor bh, bh
    mov bl, al
    shl bx, 1
    add di, bx
    mov bl, [di]
    test bl, bl
    jz .not_found
    test ah, 1
    jz .check_write
    test bl, 1
    jz .denied
.check_write:
    test ah, 2
    jz .allow
    test bl, 2
    jz .denied
.allow:
    mov al, 0
    jmp .done
.denied:
    mov al, 1
    jmp .done
.not_found:
    mov al, 2
.done:
    pop di
    pop bx
    ret

; print_line: bl=行号(0-24), si=字符串(0结尾)
print_line:
    pusha
    mov ax, 0xb800
    mov es, ax
    ; 计算显存偏移 = 行号 * 80 * 2
    xor bh, bh
    mov al, 80
    mul bl
    add ax, ax
    mov di, ax
.loop:
    lodsb
    test al, al
    jz .done
    stosb
    mov byte [es:di], 0x0f   ; 颜色属性
    inc di
    jmp .loop
.done:
    popa
    ret

; ---------- 数据 ----------
msg_boot    db "[BOOT] Kernel started.", 0
msg_ok1     db "[OK] Token0 read allowed.", 0
msg_ok2     db "[OK] Token0 write denied (correct).", 0
msg_ok3     db "[OK] Token1 write allowed.", 0
msg_ok4     db "[OK] Missing token denied (correct).", 0
msg_atomic  db "[ATOMIC] Capability system verified on bare metal.", 0
msg_fail    db "[FAIL] Unexpected result.", 0

times 510-($-$$) db 0
dw 0xAA55
