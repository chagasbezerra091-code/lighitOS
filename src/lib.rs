// src/lib.rs (Conteúdo Atualizado)

#![no_std] 
#![no_main] 
#![feature(custom_test_frameworks)] 
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)] 

use core::panic::PanicInfo;

// ------------------------------------------------------------------------
// --- Módulos do Kernel ---
// ------------------------------------------------------------------------

pub mod RustKernelConfig; 
pub mod ipc;            
pub mod drivers;        
pub mod ffi;            
pub mod interrupts;     // ⚡ NOVO: Módulo de Interrupções
pub mod memory;         // 💾 NOVO: Módulo de Memória (para referência)


// Reexporta as configurações HAL específicas da arquitetura
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/x86_64_arch.hal"]
pub mod arch_hal;

// ... (Macros print! e println! e FfiLogWriter permanecem os mesmos) ...

// ------------------------------------------------------------------------
// --- PONTO DE ENTRADA DO KERNEL (Chamado pelo bootloader.c) ---
// ------------------------------------------------------------------------

/// 🏁 O Ponto de Entrada principal do Kernel LightOS (Rust).
#[no_mangle]
pub extern "C" fn kernel_main(multiboot2_info_ptr: u64) -> ! {
    
    // ... (Inicialização do Console C++ e Logs iniciais - OMITIDOS PARA BREVIDADE) ...
    // Note: Você precisará inicializar o Console C++ com o endereço VGA real aqui!
    
    println!("----------------------------------------------------------");
    println!("LightOS: Controle transferido para kernel_main (Rust).");
    println!("Multiboot2 Info Ptr: {:#x}", multiboot2_info_ptr);
    println!("Arquitetura: {}", arch_hal::ARCH_NAME);
    println!("----------------------------------------------------------");


    // 1. INICIALIZAÇÃO DE SUBSISTEMAS CRÍTICOS (ORDEM IMPORTA)
    
    // 1.1. ⚡ Inicializar IDT e PIC (Interrupções)
    interrupts::init_idt_and_pics();
    
    // 1.2. 💾 Inicializar MMU e Heap (Ommido aqui, mas seria o próximo passo)
    // Exemplo: memory::initialize(...);
    
    // 1.3. Inicializar IPC
    ipc::initialize();

    // 2. Inicializar Drivers (Exemplo: Display)
    let display_info = drivers::display::FramebufferInfo {
        address: 0xDEADBEEF, // Placeholder real viria do Multiboot2
        width: 1024,
        height: 768,
        pitch: 1024 * 4, // 4 bytes por pixel
        bpp: 32,
    };
    
    unsafe {
        match drivers::display::DisplayDriver::new(display_info) {
            Ok(mut driver) => {
                match driver.initialize() {
                    Ok(_) => println!("[DRIVER] DisplayDriver pronto. Limpando tela..."),
                    Err(_) => println!("[ERROR] Falha ao inicializar o DisplayDriver!"),
                }
            }
            Err(_) => println!("[ERROR] Não foi possível criar o DisplayDriver."),
        }
    }
    
    // Teste de interrupção: A próxima instrução HLT só será interrompida se
    // o temporizador estiver ativo (IRQ 0) ou se houver entrada de teclado (IRQ 1).

    println!("\nLightOS Kernel rodando. Entrando em loop infinito...");
    
    loop {
        // Agora, o HLT só será desfeito por interrupções (que são tratadas pela IDT)
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

// ... (Tratamento de Pânico permanece o mesmo) ...
