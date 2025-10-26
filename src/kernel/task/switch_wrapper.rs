// src/kernel/task/switch_wrapper.rs

//! Interface de Função Estrangeira (FFI) para a Troca de Contexto em Assembly.
//! 
//! Define as stubs para as funções implementadas em `src/kernel/task/context_switch.s`.

use super::TaskContext;

// ------------------------------------------------------------------------
// --- Funções Assembly (linkadas do .s) ---
// ------------------------------------------------------------------------

extern "C" {
    /// 💾 Salva o contexto da CPU atual (registros callee-saved e RSP) no TaskContext.
    /// 
    /// Implementado em `context_switch.s`.
    /// 
    /// # Safety
    /// Inseguro devido à manipulação direta do estado da CPU.
    pub fn lightos_context_switch_save(context_ptr: *mut TaskContext);

    /// 🔄 Restaura o contexto da CPU a partir do TaskContext fornecido.
    /// * Esta função não retorna; o controle é transferido para o novo RSP/RIP.
    /// 
    /// Implementado em `context_switch.s`.
    /// 
    /// # Safety
    /// Inseguro devido à manipulação direta do estado da CPU e do RSP.
    pub fn lightos_context_switch_restore(context_ptr: *const TaskContext) -> !;
}

// ------------------------------------------------------------------------
// --- Função de Troca (Wrapper Rust) ---
// ------------------------------------------------------------------------

/// 🌉 Função Rust que gerencia a troca de contexto entre duas tarefas.
/// 
/// # Safety
/// É altamente insegura. O chamador deve garantir que o agendamento foi realizado
/// e que os ponteiros são válidos.
pub unsafe fn context_switch(
    old_context: &mut TaskContext,
    new_context: &TaskContext,
) -> ! {
    
    // 1. Salvar o contexto da tarefa atual
    lightos_context_switch_save(old_context as *mut TaskContext);

    // 2. Chamar o Scheduler (Logicamente, esta parte está no `schedule_next` do Rust)
    // Se a função 'save' retornar, significa que o contexto foi salvo, e o Rust agora
    // decide qual contexto carregar. 
    
    // 3. Restaurar o contexto da próxima tarefa (e nunca mais retornar!)
    lightos_context_switch_restore(new_context as *const TaskContext);
}
