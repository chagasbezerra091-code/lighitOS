// src/kernel/task/mod.rs

/*
 * Copyright 2017-2025 Chagas Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! Subsistema de Agendamento (Scheduling) e Gerenciamento de Tarefas para o LightOS.

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use spin::Mutex;
use x86_64::{VirtAddr, PhysAddr}; // Necessário para PhysAddr (CR3)

// Importa o VMA Manager
use crate::memory::vma::VMA_Manager;

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
    /// Endereço Físico da Tabela de Páginas de Nível 4 (CR3) desta tarefa.
    /// * Essencial para o isolamento do Userspace.
    pub cr3_phys_addr: PhysAddr,
    /// Gerenciador de Áreas de Memória Virtual do Userspace.
    pub vma_manager: VMA_Manager,
    /// Stack da tarefa (é um Box para garantir que está alocada no Heap).
    stack: Box<[u8]>,
}

/// 🆔 Tipo para o ID Único da Tarefa.
// ... (TaskId e sua implementação permanecem as mesmas) ...
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
// ... (TASK_MANAGER e initialize() permanecem os mesmos) ...
lazy_static::lazy_static! {
    /// ⚙️ Instância global do Agendador (Scheduler).
    pub static ref TASK_MANAGER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

/// 🚀 Inicializa o subsistema de agendamento (chamado do kernel_main).
pub fn initialize() {
    crate::println!("INFO: Subsistema de Agendamento inicializado.");
}

// ------------------------------------------------------------------------
// --- API Pública: Criação de Tarefas ---
// ------------------------------------------------------------------------

/// ➕ Cria e agenda uma nova tarefa.
/// 
/// O `cr3_base` deve ser o endereço físico da P4 Table desta tarefa.
pub fn spawn_task(entry_point: extern "C" fn(), cr3_base: PhysAddr) {
    // 1. Aloca uma stack
    let stack_size: usize = 4096;
    let stack = {
        let buf = vec![0u8; stack_size];
        buf.into_boxed_slice()
    };
    
    // 2. Define o ponteiro da stack
    let stack_top = VirtAddr::from_ptr(stack.as_ptr()) + stack_size;
    
    // 3. Cria o Contexto
    let context = TaskContext::new(stack_top, entry_point as u64);

    // 4. Cria a Estrutura da Tarefa
    let new_task = Task {
        id: TaskId::new(),
        context,
        cr3_phys_addr: cr3_base, // Endereço da P4 Table da nova tarefa
        vma_manager: VMA_Manager::new(), // Um novo gerenciador de VMA para isolamento
        stack,
    };
    
    // 5. Adiciona a Tarefa ao Agendador
    TASK_MANAGER.lock().add_task(new_task);
    crate::println!("INFO: Tarefa #{} agendada. (CR3: {:#x})", 
        new_task.id.0, new_task.cr3_phys_addr.as_u64());
}

// ------------------------------------------------------------------------
// --- Função de Alternância de Contexto (Chamada pelo Timer IRQ) ---
// ------------------------------------------------------------------------

/// 🔄 Função principal de pré-empting (alternância de contexto).
/// 
/// # Safety
/// É insegura; exige manipulação de registradores CR3 e troca de contexto.
pub unsafe fn schedule_next(current_context: &mut TaskContext) {
    TASK_MANAGER.lock().schedule_next(current_context);
}
