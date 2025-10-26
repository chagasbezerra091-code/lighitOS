// src/kernel/task/context_switch.s

// ------------------------------------------------------------------------
// --- Definições Globais e Convenções ---
// ------------------------------------------------------------------------

.global lightos_context_switch_save
.global lightos_context_switch_restore

// Convenção de Chamada x86_64:
// Argumento 1: %rdi
// Argumento 2: %rsi
// Argumento 3: %rdx

// Estrutura do TaskContext (Deve corresponder EXATAMENTE a src/kernel/task/context.rs)
// struct TaskContext {
//   rflags: u64,
//   rbx: u64,
//   rbp: u64,
//   r12: u64,
//   r13: u64,
//   r14: u64,
//   r15: u64,
//   rsp: u64, // O último campo na estrutura Rust/C++
// }
.equ CONTEXT_RFLAGS, 0
.equ CONTEXT_RBX,    8
.equ CONTEXT_RBP,    16
.equ CONTEXT_R12,    24
.equ CONTEXT_R13,    32
.equ CONTEXT_R14,    40
.equ CONTEXT_R15,    48
.equ CONTEXT_RSP,    56
.equ CONTEXT_SIZE,   64  // 8 registradores * 8 bytes/registrador

.section .text

// ------------------------------------------------------------------------
// --- lightos_context_switch_save (Salvar o Contexto Atual) ---
// ------------------------------------------------------------------------

// fn lightos_context_switch_save(context_ptr: *mut TaskContext);
// Argumento 1: %rdi contém o endereço do TaskContext a ser preenchido.

lightos_context_switch_save:
    // Salvar o RFLAGS (Interrupts são desabilitados via timer_interrupt_handler)
    pushfq
    pop %rax
    mov %rax, CONTEXT_RFLAGS(%rdi)

    // Salvar Registradores Voláteis (callee-saved)
    // O Rust espera que esses registradores sejam preservados nas chamadas de função.
    // Em um kernel, é mais seguro salvar todos.
    mov %rbx, CONTEXT_RBX(%rdi)
    mov %rbp, CONTEXT_RBP(%rdi)
    mov %r12, CONTEXT_R12(%rdi)
    mov %r13, CONTEXT_R13(%rdi)
    mov %r14, CONTEXT_R14(%rdi)
    mov %r15, CONTEXT_R15(%rdi)
    
    // Salvar o Stack Pointer (RSP)
    // O RSP (R0) aponta para a pilha do kernel/thread atual.
    mov %rsp, CONTEXT_RSP(%rdi)
    
    // O RIP (Instruction Pointer) é salvo implicitamente pela chamada da interrupção.
    // O ponto de retorno (RIP) é salvo como o endereço da próxima instrução após
    // a interrupção.

    ret // Retorna ao chamador (Rust/C)

// ------------------------------------------------------------------------
// --- lightos_context_switch_restore (Restaurar o Contexto) ---
// ------------------------------------------------------------------------

// fn lightos_context_switch_restore(context_ptr: *const TaskContext);
// Argumento 1: %rdi contém o endereço do TaskContext a ser carregado.

lightos_context_switch_restore:
    // Carregar o Stack Pointer (RSP) da nova tarefa
    mov CONTEXT_RSP(%rdi), %rsp

    // Carregar Registradores Voláteis
    mov CONTEXT_RBX(%rdi), %rbx
    mov CONTEXT_RBP(%rdi), %rbp
    mov CONTEXT_R12(%rdi), %r12
    mov CONTEXT_R13(%rdi), %r13
    mov CONTEXT_R14(%rdi), %r14
    mov CONTEXT_R15(%rdi), %r15
    
    // Carregar o RFLAGS
    mov CONTEXT_RFLAGS(%rdi), %rax
    push %rax
    popfq

    // Carregar o RIP (Instrução Pointer)
    // Se a tarefa for nova, o RIP deve ser definido como o Entry Point.
    // O TaskContext::new define o entry point em R15.
    // Neste ponto, o RSP aponta para a stack da nova tarefa.
    // O código de interrupção em Assembly (ou o Wrapper FFI) deve garantir
    // que o RIP (e o Stack Frame de Interrupção) seja reconstruído.

    // Já que este código é chamado após a lógica Rust, a maneira mais simples
    // é fazer um 'ret' e usar o endereço salvo na stack.
    // Para iniciar uma nova thread, precisamos que o RIP aponte para a função.
    
    // Método 1: Se a função de entrada estiver em R15 (como definido em context.rs)
    // mov %r15, %rax  // Carrega o entry point
    // jmp *%rax     // Pula para a função de entrada

    // Método 2: O método 'ret' (que é para restaurar de uma chamada de função)
    // Vamos usar a convenção de que o RSP carregado já aponta para um endereço de retorno válido.
    
    ret // Retorna ao endereço de retorno salvo na stack da nova thread.
