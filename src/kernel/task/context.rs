// src/kernel/task/context.rs

//! Estrutura que define o Contexto de Execução (Task Context) para uma thread.
//! 
//! Contém todos os registradores que precisam ser salvos e restaurados durante
//! a troca de contexto.

use x86_64::VirtAddr;

/// 🧠 O Contexto de Execução (Conjunto de Registradores).
/// * Os campos devem refletir exatamente o que o código Assembly de troca de contexto 
/// * espera salvar/restaurar.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)] // Garante a ordem e o layout C (crucial para Assembly)
pub struct TaskContext {
    // Ordem de salvamento/restauração dos registradores:
    pub rflags: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64, // Stack Pointer (último a ser restaurado)
    
    // NOTA: O RIP (Instruction Pointer) é manipulado separadamente
    // ou como parte de uma Interrupção Stack Frame.
}

impl TaskContext {
    /// 🏭 Cria um contexto inicial para uma nova tarefa.
    /// * O contexto é manipulado para simular o retorno de uma interrupção.
    pub fn new(stack_top: VirtAddr, entry_point_addr: u64) -> Self {
        let mut context = TaskContext::default();
        
        // 1. Configurar o Stack Pointer (RSP)
        // A stack de contexto deve apontar para onde o código Assembly
        // esperaria encontrar o contexto inicial (incluindo o endereço de retorno/entry point).
        
        // Simulação do Stack Frame da Interrupção:
        // O Rust/C++ espera que o rsp aponte para o topo da TaskContext.
        context.rsp = stack_top.as_u64();
        
        // O RIP deve ser definido como o endereço da função de entrada.
        // Já que a troca de contexto é feita via Assembly (que lida com RIP),
        // este endereço de entrada é armazenado temporariamente em RBX/RCX
        // ou escrito diretamente no topo da stack.
        
        // Para simplificar, armazenamos o entry point em um registrador que será
        // copiado para o RIP pelo assembly (ex: R15).
        context.r15 = entry_point_addr;
        
        // RFLAGS: Habilitar interrupções (bit 9 setado)
        context.rflags = 0x202; // Bit 1 (sempre setado) + Bit 9 (Interrupt Enable)
        
        context
    }
}
