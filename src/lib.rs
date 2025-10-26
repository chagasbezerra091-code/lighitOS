// src/lib.rs

/*
 * Copyright 2024 Chagas Inc.
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

#![no_std] 
#![no_main] 
#![feature(custom_test_frameworks)] 
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)] 

extern crate alloc; // Necessário para o Heap e o Scheduler

use core::panic::PanicInfo;
use x86_64::{VirtAddr, PhysAddr};

// ------------------------------------------------------------------------
// --- Módulos do Kernel ---
// ------------------------------------------------------------------------

pub mod RustKernelConfig; 
pub mod ipc;            
pub mod drivers;        
pub mod ffi;            
pub mod interrupts;     // IDT, PIC e Handlers IRQ
pub mod memory;         // MMU, Paging e Heap
pub mod task;           // Scheduler e Context Switch
pub mod syscall;        // Dispatcher de Chamadas de Sistema


// Reexporta as configurações HAL específicas da arquitetura
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/x86_64_arch.hal"]
pub mod arch_hal;

// ------------------------------------------------------------------------
// --- DEFINIÇÕES DE MEMÓRIA (Constantes para Inicialização) ---
// ------------------------------------------------------------------------

/// Tamanho do Heap do Kernel (512 KB)
const HEAP_SIZE: usize = 512 * 1024;
/// Endereço virtual onde o Heap do Kernel deve começar
const KERNEL_HEAP_START: VirtAddr = VirtAddr::new_truncate(0xFFFF_8000_0100_0000); 

// ------------------------------------------------------------------------
// --- PONTO DE ENTRADA DO KERNEL ---
// ------------------------------------------------------------------------

/// 🏁 O Ponto de Entrada principal do Kernel LightOS (Rust).
#[no_mangle]
pub extern "C" fn kernel_main(multiboot2_info_ptr: u64) -> ! {
    
    // ... (Inicialização do Console C++ e Logs iniciais - OMITIDOS PARA BREVIDADE) ...
    println!("----------------------------------------------------------");
    println!("LightOS Kernel: Controle transferido (Rust).");
    println!("Arquitetura: {}", arch_hal::ARCH_NAME);
    println!("----------------------------------------------------------");

    // 1. INICIALIZAÇÃO CRÍTICA (ORDEM É VITAL)
    
    // 1.1. ⚡ Inicializar IDT, PIC e Habilitar Interrupções
    interrupts::init_idt_and_pics();
    
    // 1.2. 💾 Inicializar Paging e Heap
    let pmm_allocator = memory::frame_alloc::PhysicalMemoryManager::new();

    match unsafe { 
        memory::paging::init_paging_and_heap(
            multiboot2_info_ptr, 
            pmm_allocator, 
            KERNEL_HEAP_START, 
            HEAP_SIZE
        )
    } {
        Ok(_) => println!("[MMU] Paging e Heap inicializados com sucesso."),
        Err(e) => {
            println!("[FATAL] Falha na inicialização da Memória: {:?}", e);
            loop { unsafe { x86_64::instructions::hlt(); } }
        }
    }
    
    // 1.3. ⚙️ Inicializar Subsistemas Essenciais
    ipc::initialize();
    syscall::initialize();
    task::initialize(); // Inicializa o Scheduler/Task Manager
    

    // 2. Inicializar Drivers e Iniciar Tarefas
    
    // 2.1. Drivers (Exemplo: Display)
    // O Driver de Display usará o Heap e o Paging (que agora estão prontos)
    // ... (Código de inicialização do DisplayDriver permanece o mesmo) ...
    println!("[DRIVER] Drivers básicos inicializados.");

    // 2.2. Iniciar Tarefas de Usuário (Exemplo)
    // task::spawn_task(userspace_entry); // Uma função FFI de userspace
    
    println!("\nLightOS Kernel pronto. Entrando em loop IDLE, aguardando IRQs...");
    
    // Loop principal do Kernel (A tarefa IDLE/Kernel Task 0)
    loop {
        // O HLT será interrompido pelo temporizador (IRQ0), que acionará o Scheduler
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

// ------------------------------------------------------------------------
// --- Tratamento de Pânico (Panic Handler) e Testes ---
// ------------------------------------------------------------------------

// ... (Panic Handler e Test Runner permanecem os mesmos) ...
