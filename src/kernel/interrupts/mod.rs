// src/kernel/interrupts/mod.rs

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

//! Subsistema de Gerenciamento de Interrupções e Exceções para o LightOS.

use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};
use x86_64::registers::control::Cr2;
use lazy_static::lazy_static; 
use spin::Mutex; 
use x86_64::instructions::interrupts; // Necessário para desabilitar/reabilitar IRQ

// Módulos internos
pub mod pic;
use crate::{task, syscall}; 
use crate::memory::vma::VMA_Error; // Importa o erro VMA

// ... (Constantes e Enumerações InterruptIndex permanecem as mesmas) ...
// ... (Funções lightos_* Assembly e init_idt_and_pics permanecem as mesmas) ...
// ... (Exceções: divide_error_handler, double_fault_handler, general_protection_fault_handler permanecem as mesmas) ...

// ------------------------------------------------------------------------
// --- Handler de Falha de Página (Page Fault) ---
// ------------------------------------------------------------------------

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // 1. Obter o endereço virtual que causou a falha (CR2)
    let fault_addr = Cr2::read();
    
    // Obter o ID da tarefa atual (para logs e contexto)
    let current_task_id = task::TASK_MANAGER.lock().current_task.as_ref()
        .map_or(0, |t| t.id.0);

    crate::println!("\n--- [PAGE FAULT] Tarefa #{} ---", current_task_id);
    crate::println!("Endereço que Falhou (CR2): {:?}", fault_addr);
    crate::println!("Código de Erro: {:?}", error_code);
    
    // 2. Tentar resolver a falha usando o VMA Manager da tarefa atual
    let resolution_result = {
        // Bloqueia o Scheduler para acessar o VMA Manager da tarefa atual
        let mut scheduler = task::TASK_MANAGER.lock();
        
        match scheduler.current_task.as_mut() {
            Some(task) => {
                // Tenta mapear a página. Isso implementa o Demand Paging.
                task.vma_manager.map_vma_page(fault_addr)
            }
            None => {
                // Se não há tarefa atual (acontece antes do Scheduler iniciar)
                Err(VMA_Error::NoAreaFound)
            }
        }
    };
    
    // 3. Avaliar o resultado da tentativa de resolução
    match resolution_result {
        Ok(_) => {
            // A falha foi resolvida (a página foi mapeada sob demanda).
            // O retorno da interrupção (IRETQ) fará com que a CPU tente a 
            // instrução falha novamente, que agora deve ter sucesso.
            crate::println!("INFO: Falha de Página resolvida (Demand Paging).");
        }
        Err(VMA_Error::NoAreaFound) => {
            // Se não for um endereço válido em nenhum VMA, é uma violação de acesso.
            crate::println!("FATAL: Endereço {:#x} não pertence a nenhuma VMA válida. Matando Tarefa.", fault_addr.as_u64());
            // Ação: Terminar a tarefa atual ou entrar em pânico se for o Kernel.
            // Para simplificar, causamos um pânico no kernel por agora:
            
            crate::println!("Stack Frame: {:#?}", stack_frame);
            loop { x86_64::instructions::hlt(); }
        }
        Err(VMA_Error::OOM) => {
            crate::println!("FATAL: Falha de Página (OOM) - Memória física esgotada.");
            loop { x86_64::instructions::hlt(); }
        }
        Err(e) => {
            crate::println!("FATAL: Erro desconhecido na VMA: {:?}", e);
            loop { x86_64::instructions::hlt(); }
        }
    }
}

// ------------------------------------------------------------------------
// --- Handlers de IRQ (lightos_timer_handler_rust, lightos_keyboard_handler_rust) ---
// ... (Permanecem os mesmos) ...
// ------------------------------------------------------------------------
