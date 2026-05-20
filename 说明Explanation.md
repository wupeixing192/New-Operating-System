这是一个最简单的原子回路证明，由于作者当前环境为ubuntu，环境不稳定，况且出现许多错误，已经折腾了8小时，于是我决定：用16位实模式汇编证明在裸机（qemu）中验证原子回路：

This is the simplest atomic loop proof. Since the author's current environment is Ubuntu, which is unstable, and many errors have occurred, after struggling for 8 hours, I decided to: use 16-bit real mode assembly to prove and verify the atomic loop on bare metal (qemu):

在boot.asm文件中，你会看到：
In the boot.asm file, you will see:



; ---------- 数据 ----------
msg_boot    db "[BOOT] Kernel started.", 0
msg_ok1     db "[OK] Token0 read allowed.", 0
msg_ok2     db "[OK] Token0 write denied (correct).", 0
msg_ok3     db "[OK] Token1 write allowed.", 0
msg_ok4     db "[OK] Missing token denied (correct).", 0
msg_atomic  db "[ATOMIC] Capability system verified on bare metal.", 0
msg_fail    db "[FAIL] Unexpected result.", 0



对不熟悉汇编的人来说，看到 db 定义的一串字符串，确实容易误以为只是静态打印，没有体现动态验证。但事实上，这段代码根据 use_token 的返回值，动态选择打印 msg_ok1 还是 msg_fail，这才是核心。

For people not familiar with assembly, seeing a string defined by db can easily lead to the mistaken belief that it is just static printing, without reflecting dynamic validation. But in fact, this code dynamically chooses to print msg_ok1 or msg_fail based on the return value of use_token, which is the core.

证明/Proof：

故意破坏一个令牌的权限，观察输出是否从 [OK] 变为 [FAIL]。

Intentionally damage the permissions of a token and observe whether the output changes from [OK] to [FAIL].

例如，把令牌0的权限从 0b01（只读）改为 0b00（无权限），那么测试1（令牌0读）就会失败，输出会从 [OK] Token0 read allowed. 变为 [FAIL] Unexpected result. 这就有力地证明了输出是动态的。

For example, if the permissions of token 0 are changed from 0b01 (read-only) to 0b00 (no permission), then Test 1 (read token 0) will fail, and the output will change from [OK] Token0 read allowed. to [FAIL] Unexpected result. This strongly proves that the output is dynamic.

具体修改：在 boot.asm 中找到下面这一行：

Specific modification: Find the following line in boot.asm:


mov byte [0x7e00], 0b01   ; 令牌0: 只读/Token 0: Read-only


改为/Change to：


mov byte [0x7e00], 0b00   ; 令牌0: 无权限/Token 0: No permission



重新编译运行，你会看到第一项测试输出 [FAIL] Unexpected result.，而其它测试保持不变。这就能直观地说明：程序在运行时根据令牌的实际权限做出了不同的决策，而不是静态打印。

Recompile and run, and you will see that the first test outputs [FAIL] Unexpected result., while the other tests remain unchanged. This clearly illustrates that the program makes different decisions at runtime based on the actual permissions of the token, rather than printing statically.


代码中动态决策的关键点/Key points of dynamic decision-making in code：

测试1的代码片段/Code snippet for Test 1：


call use_token       ; 调用权限检查，返回值在 al/Call permission check, return value is in al
cmp al, 0            ; 比较是否允许/Compare whether allowed
je .t1_ok            ; 如果允许，跳转到 .t1_ok，打印 msg_ok1/If allowed, jump to .t1_ok and print msg_ok1
mov si, msg_fail     ; 否则，打印 msg_fail/Otherwise, print msg_fail



use_token 函数内部会读取令牌表（[0x7e00] 等），根据请求的读写位和实际权限位返回不同的值（0=允许，1=权限不足，2=未找到）。

The use_token function internally reads the token table ([0x7e00], etc.) and returns different values (0=allowed, 1=insufficient permissions, 2=not found) based on the requested read/write bit and the actual permission bit.

所以，尽管字符串本身是静态的，但选择哪一条字符串是由运行时动态逻辑决定的。这正是“能力令牌验证”的含义。

Therefore, although the string itself is static, which string is chosen is determined by runtime dynamic logic. This is precisely the meaning of 'capability token verification'.


