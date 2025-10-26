// src/kernel/task/scheduler.rs

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

//! Implementação do Algoritmo de Agendamento (Round-Robin) com isolamento de memória.

use alloc::collections::VecDeque;
use super::{Task, TaskContext};
use x86_64::registers::control::Cr3;
use x86_64::PhysAddr;

/// 🔄 O Agendador de Tarefas (Round-Robin).
pub struct Scheduler {
    /// Fila de tarefas prontas para serem executadas (Task Ready Queue).
    task_queue: VecDeque<Task>,
    /// A tarefa atualmente em execução.
    current_task: Option<Task>,
}

impl Scheduler {
    /// 🏭 Cria um novo Agendador.
    pub fn new() -> Self {
        Scheduler {
            task_queue: VecDeque::new(),
            current_task: None,
        }
    }

    /// ➕ Adiciona uma tarefa à fila de prontas.
    pub fn add_task(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }

    /// 🔄 Implementa a lógica do agendamento (Round-Robin) e realiza a troca de CR3.
    /// * Escolhe a próxima tarefa, salva o contexto da atual e prepara para a troca.
    ///
    /// # Safety
    /// `current_context` é o contexto salvo da tarefa que acabou de ser pré-emptada.
    pub unsafe fn schedule_next(&mut self, current_context: &mut TaskContext) {
        
        // 1. Lidar com a primeira execução (Kernel Task 0)
        if self.current_task.is_none() {
            // Captura o endereço CR3 atual do Kernel (P4 Table)
            let (p4_table_frame, _) = Cr3::read();
            
            // Cria a "tarefa" para o kernel_main (Task ID 0).
            // NOTA: Para simplificar, o VMA Manager é vazio.
            let kernel_task = Task {
                id: super::TaskId(0), 
                context: *current_context,
                cr3_phys_addr: p4_table_frame.start_address(), // CR3 do Kernel
                vma_manager: crate::memory::vma::VMA_Manager::new(), 
                stack: Box::new([]), 
            };
            self.current_task = Some(kernel_task);
        }

        // 2. Pré-emptar a tarefa atual: Salvar o contexto dela e colocá-la no final da fila.
        // Já que esta função é chamada do Assembly Wrapper (após lightos_context_switch_save), 
        // o `current_context` já contém os registradores salvos (RBX, RBP, R12-R15, RFLAGS).
        if let Some(mut prev_task) = self.current_task.take() {
            prev_task.context = *current_context;
            self.task_queue.push_back(prev_task);
        }

        // 3. Selecionar a próxima tarefa (Round-Robin)
        if let Some(next_task) = self.task_queue.pop_front() {
            
            let next_task_id = next_task.id.0;
            let next_cr3 = next_task.cr3_phys_addr;
            
            // 4. Trocar o Contexto de Memória (CR3)
            // Esta é a parte crucial para o isolamento.
            if Cr3::read().0.start_address() != next_cr3 {
                Self::switch_cr3(next_cr3);
                crate::println!("SCHED: Troca de CR3 para {:#x}", next_cr3.as_u64());
            }

            // 5. Restaurar o Contexto da CPU (Feito pelo Assembly, aqui preparamos o contexto)
            *current_context = next_task.context; 
            self.current_task = Some(next_task);
            
            crate::println!("SCHED: Trocando para Tarefa #{}", next_task_id);

        } else {
            // Nenhuma tarefa pronta (Retorna para a tarefa IDLE do Kernel)
            crate::println!("SCHED: Nenhuma tarefa de usuário disponível. Continuar IDLE.");
            // O contexto do kernel_main será restaurado.
        }
    }
    
    /// ⚛️ Troca o registro CR3 da CPU.
    /// 
    /// # Safety
    /// Altera o mapa de memória global, deve ser chamado apenas pelo Scheduler.
    fn switch_cr3(p4_addr: PhysAddr) {
        let frame = x86_64::structures::paging::PhysFrame::containing_address(p4_addr);
        
        // Troca o CR3, efetivamente trocando o Page Table usado pela CPU.
        // O `read_and_disable_cache` é a maneira x86_64-library de garantir a troca.
        unsafe {
            Cr3::write(frame, Cr3::read().1);
        }
    }
}
