// src/kernel/task/scheduler.rs

//! Implementação do Algoritmo de Agendamento (Round-Robin).

use alloc::collections::VecDeque;
use super::{Task, TaskContext};

/// 🔄 O Agendador de Tarefas (Round-Robin).
pub struct Scheduler {
    /// Fila de tarefas prontas para serem executadas (Task Ready Queue).
    task_queue: VecDeque<Task>,
    /// A tarefa atualmente em execução.
    current_task: Option<Task>,
    /// ID da próxima tarefa a ser despachada.
    next_task_id: Option<u64>,
}

impl Scheduler {
    /// 🏭 Cria um novo Agendador.
    pub fn new() -> Self {
        Scheduler {
            task_queue: VecDeque::new(),
            current_task: None,
            next_task_id: None,
        }
    }

    /// ➕ Adiciona uma tarefa à fila de prontas.
    pub fn add_task(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    /// 🔄 Implementa a lógica do agendamento (Round-Robin).
    /// * Escolhe a próxima tarefa e prepara para a troca de contexto.
    ///
    /// # Safety
    /// `current_context` é o contexto salvo da tarefa que acabou de ser pré-emptada.
    pub unsafe fn schedule_next(&mut self, current_context: &mut TaskContext) {
        
        // 1. Se não há tarefa atual (primeira execução), usa o contexto do kernel_main
        if self.current_task.is_none() {
            // Cria uma "tarefa" para o kernel_main (para salvar seu contexto).
            let kernel_task = Task {
                id: super::TaskId(0), // ID 0 é a tarefa do Kernel
                context: *current_context,
                stack: Box::new([]), // Sem stack separada
            };
            self.current_task = Some(kernel_task);
        }

        // 2. Pré-emptar a tarefa atual: Salvar o contexto dela e colocá-la no final da fila.
        if let Some(mut prev_task) = self.current_task.take() {
            prev_task.context = *current_context;
            self.task_queue.push_back(prev_task);
        }

        // 3. Selecionar a próxima tarefa (Round-Robin: Pega a primeira da fila)
        if let Some(next_task) = self.task_queue.pop_front() {
            
            // 4. Trocar para a próxima tarefa
            self.next_task_id = Some(next_task.id.0);
            *current_context = next_task.context; // Restaura o contexto da nova tarefa
            self.current_task = Some(next_task);
            
            crate::println!("SCHED: Trocando para Tarefa #{}", self.current_task.as_ref().unwrap().id.0);

        } else {
            // Nenhuma tarefa pronta (isso não deve acontecer em um sistema real, 
            // a tarefa IDLE sempre deve estar na fila).
            crate::println!("SCHED: Nenhuma tarefa disponível. Voltando para o kernel_main (Idle).");
            // Se o kernel_main foi salvo na fila, ele será executado aqui.
        }
    }
}
