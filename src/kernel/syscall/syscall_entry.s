// src/kernel/syscall/syscall_entry.s

// ------------------------------------------------------------------------
// --- Definições Globais e Convenções ---
// ------------------------------------------------------------------------

.global lightos_syscall_entry

// Convenção de Chamada Syscall (x86_64 - Linux/Padrão Comum):
// Syscall ID: RAX
// Argumentos: RDI, RSI, RDX, R10, R8, R9
// Retorno:    RAX

// ------------------------------------------------------------------------
// --- lightos_syscall_entry (Chamado pela IDT ou Instrução SYSCALL) ---
// ------------------------------------------------------------------------

// Esta rotina deve ser configurada na IDT (vetor INT 0x80) ou como o 
// handler MSR para a instrução SYSCALL.

lightos_syscall_entry:
    // 1. Salvar Registradores Voláteis (caller-saved)
    // Precisamos salvar os registradores que serão usados pela função Rust 
    // e que o Userspace espera que sejam preservados ou contenham dados.
    pushq %rcx      // RCX (usado para salvar o RIP no SYSCALL/SYSRET)
    pushq %r11      // R11 (usado para salvar o RFLAGS no SYSCALL/SYSRET)
    pushq %rdi      // Arg 1
    pushq %rsi      // Arg 2
    pushq %rdx      // Arg 3
    pushq %r10      // Arg 4
    pushq %r8       // Arg 5
    pushq %r9       // Arg 6
    pushq %rax      // Syscall ID (RAX)

    // 2. Preparar Argumentos para a função Rust:
    // A função Rust é: syscall_dispatcher(id: u64, args: SyscallArgs) -> u64
    
    // Arg 1 (id): %rdi = RAX (já está no topo da stack, vamos movê-lo)
    mov %rsp, %rdi      // Ponteiro para o RAX salvo (Syscall ID)
    mov (%rdi), %rdi    // %rdi = Syscall ID (valor de RAX)

    // Arg 2 (args): %rsi = Ponteiro para a estrutura SyscallArgs.
    // Usaremos a stack para montar a estrutura SyscallArgs:
    
    // Alocar espaço na stack para a estrutura SyscallArgs (6 * 8 bytes = 48 bytes)
    sub $48, %rsp 
    
    // Mover os argumentos salvos da stack para a estrutura SyscallArgs na nova área da stack:
    // SyscallArgs fields: arg1-RDI, arg2-RSI, arg3-RDX, arg4-R10, arg5-R8, arg6-R9
    
    // Endereço de RAX (Syscall ID) na stack: %rdi_ptr = %rsp + 48
    // Endereço base da nova SyscallArgs: %rsp
    
    // Args na stack (do topo para baixo): RAX, R9, R8, R10, RDX, RSI, RDI, R11, RCX
    // Offset em relação ao topo da SyscallArgs (NOVA stack):
    
    mov 56(%rsp), %rax  // RAX (Syscall ID) - Temporariamente
    mov 48(%rsp), %r9   // R9 (Arg 6)
    mov 40(%rsp), %r8   // R8 (Arg 5)
    mov 32(%rsp), %r10  // R10 (Arg 4)
    mov 24(%rsp), %rdx  // RDX (Arg 3)
    mov 16(%rsp), %rsi  // RSI (Arg 2)
    mov 8(%rsp), %rdi_val // RDI (Arg 1) - Renomeado para evitar conflito com %rdi do Rust
    
    // Preenche a estrutura SyscallArgs na stack (%rsp):
    mov %rdi_val, 0(%rsp)   // arg1 (RDI)
    mov %rsi, 8(%rsp)       // arg2 (RSI)
    mov %rdx, 16(%rsp)      // arg3 (RDX)
    mov %r10, 24(%rsp)      // arg4 (R10)
    mov %r8, 32(%rsp)       // arg5 (R8)
    mov %r9, 40(%rsp)       // arg6 (R9)
    
    // Arg 1 do Rust (%rdi): Syscall ID (já preenchido acima)
    // Arg 2 do Rust (%rsi): Ponteiro para SyscallArgs (%rsp)
    mov %rsp, %rsi

    // 3. Chamada para o Dispatcher Rust
    call syscall_dispatcher 

    // O valor de retorno (RAX) da função Rust já está em %rax.

    // 4. Limpar a Stack e Restaurar Registradores
    
    // Desalocar a SyscallArgs (48 bytes)
    add $48, %rsp 

    // Restaurar registradores salvos (ignora RAX que contém o retorno)
    popq %rax       // Restaura RAX (agora é o valor de retorno)
    popq %r9        
    popq %r8
    popq %r10
    popq %rdx
    popq %rsi
    popq %rdi
    popq %r11
    popq %rcx

    // 5. Retorno ao Userspace
    // Em um sistema SYSCALL/SYSRET, a instrução seria SYSRET
    // Em um sistema INT, seria IRETQ
    sysretq // Instrução de retorno otimizada para Userspace
