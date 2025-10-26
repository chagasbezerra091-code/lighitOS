// src/kernel/task/mod.rs

//! Subsistema de Agendamento (Scheduling) e Gerenciamento de Tarefas para o LightOS.
//! 
//! Responsável por criar e alternar contextos de execução de tarefas.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use spin::Mutex;
use x86_64::VirtAddr;

mod context;
mod scheduler;

pub use context::TaskContext;
pub use scheduler::Scheduler;

// ------------------------------------------------------------------------
// --- Estrutura de Tarefa (Task) ---
// ------------------------------------------------------------------------

/// 🧵 Estado de uma Tarefa (Task/Thread) no LightOS.
pub struct Task {
    /// ID único da tarefa.
    id: TaskId,
    /// O contexto de registradores da CPU a ser salvo/restaurado.
    pub context: TaskContext,
    /// Stack da tarefa (é um Box para garantir que está alocada no Heap).
    stack: Box<[u8]>,
}

/// 🆔 Tipo para o ID Único da Tarefa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskId(u64);

impl TaskId {
    /// Gera um novo ID único (incrementado estaticamente).
    pub fn new() -> TaskId {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

// ------------------------------------------------------------------------
// --- Gerenciador Global de Tarefas ---
// ------------------------------------------------------------------------

lazy_static::lazy_static! {
    /// ⚙️ Instância global do Agendador (Scheduler).
    /// Deve ser acessado através de um Mutex para garantir exclusividade.
    pub static ref TASK_MANAGER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

/// 🚀 Inicializa o subsistema de agendamento (chamado do kernel_main).
/// * Cria a tarefa IDLE (inatividade).
pub fn initialize() {
    // 1. Cria a primeira tarefa (tarefa IDLE, que é o loop hlt no kernel_main)
    // O Scheduler assumirá que o kernel_main é a primeira tarefa (Task 0).

    crate::println!("INFO: Subsistema de Agendamento inicializado.");
}

// ------------------------------------------------------------------------
// --- API Pública ---
// ------------------------------------------------------------------------

/// ➕ Cria e agenda uma nova tarefa.
/// * `entry_point`: O endereço da função a ser executada pela tarefa.
pub fn spawn_task(entry_point: extern "C" fn()) {
    // 1. Aloca uma stack para a nova tarefa (Ex: 4KB)
    let stack_size: usize = 4096;
    let stack = {
        // Aloca um array de bytes no Heap.
        let buf = vec![0u8; stack_size];
        buf.into_boxed_slice()
    };
    
    // 2. Define o ponteiro da stack no topo (para x86_64, a stack cresce para baixo)
    let stack_top = VirtAddr::from_ptr(stack.as_ptr()) + stack_size;
    
    // 3. Cria o Contexto da Tarefa (simulando a primeira interrupção/restauração)
    let context = TaskContext::new(stack_top, entry_point as u64);

    // 4. Cria a Estrutura da Tarefa
    let new_task = Task {
        id: TaskId::new(),
        context,
        stack,
    };
    
    // 5. Adiciona a Tarefa ao Agendador
    TASK_MANAGER.lock().add_task(new_task);
    crate::println!("INFO: Tarefa #{} agendada para execução.", new_task.id.0);
}

// ------------------------------------------------------------------------
// --- Função de Alternância de Contexto (Chamada pelo Timer IRQ) ---
// ------------------------------------------------------------------------

/// 🔄 Função principal de pré-empting (alternância de contexto).
/// * Chamada pelo `timer_interrupt_handler` do módulo `interrupts`.
/// 
/// # Safety
/// Esta função é insegura pois exige a manipulação de registradores e a 
/// troca de contexto do kernel. Deve ser chamada apenas em contexto de interrupção.
pub unsafe fn schedule_next(current_context: &mut TaskContext) {
    TASK_MANAGER.lock().schedule_next(current_context);
}
