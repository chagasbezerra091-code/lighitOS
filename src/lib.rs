// src/lib.rs (Conteúdo Atualizado - Versão Final de Inicialização)

#![no_std] 
#![no_main] 
// ... (outras features) ...
#![allow(dead_code)] 

use core::panic::PanicInfo;
use x86_64::{VirtAddr, PhysAddr}; // Tipos importantes para a memória

// ------------------------------------------------------------------------
// --- Módulos do Kernel ---
// ------------------------------------------------------------------------

pub mod RustKernelConfig; 
pub mod ipc;            
pub mod drivers;        
pub mod ffi;            
pub mod interrupts;     
pub mod memory;         // Contém Paging e Heap

// Reexporta as configurações HAL específicas da arquitetura
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/x86_64_arch.hal"]
pub mod arch_hal;

// ------------------------------------------------------------------------
// --- DEFINIÇÕES DE MEMÓRIA (Constantes para Inicialização) ---
// ------------------------------------------------------------------------

/// Tamanho do Heap do Kernel (512 KB)
const HEAP_SIZE: usize = 512 * 1024;
/// Endereço virtual onde o Heap do Kernel deve começar (Baseado no HAL)
// Em um kernel real, este endereço viria do mapeamento definido no código Assembly/C.
const KERNEL_HEAP_START: VirtAddr = VirtAddr::new_truncate(0xFFFF_8000_0100_0000); 
// Assumindo que KERNEL_HH_BASE é 0xFFFF_8000_0000_0000 e o Heap começa em +16MB.

// ------------------------------------------------------------------------
// ... (Macros print! e println! e FfiLogWriter permanecem os mesmos) ...
// ------------------------------------------------------------------------

// ------------------------------------------------------------------------
// --- PONTO DE ENTRADA DO KERNEL (Chamado pelo bootloader.c) ---
// ------------------------------------------------------------------------

/// 🏁 O Ponto de Entrada principal do Kernel LightOS (Rust).
#[no_mangle]
pub extern "C" fn kernel_main(multiboot2_info_ptr: u64) -> ! {
    
    // 1. INICIALIZAÇÃO TEMPORÁRIA DO CONSOLE (C++)
    // Nota: O endereço VGA_TEXT_BUFFER_ADDR viria de RustKernelConfig::arch_hal
    // Assumimos que a inicialização do C++ é feita via FFI ou código Assembly.
    let vga_addr = 0xb8000 as usize; // Endereço VGA de 16-bit
    
    // Este é um placeholder, a implementação real do FFI deve chamar Console::initialize
    unsafe {
        // Assume-se que esta função C existe e inicializa o Console C++
        // lightos_driver_console_init(vga_addr as u64);
    }
    
    println!("----------------------------------------------------------");
    println!("LightOS: Controle transferido para kernel_main (Rust).");
    println!("Arquitetura: {}", arch_hal::ARCH_NAME);
    println!("----------------------------------------------------------");

    // 2. INICIALIZAÇÃO CRÍTICA (ORDEM É VITAL)
    
    // 2.1. ⚡ Inicializar IDT e PIC (Interrupções)
    interrupts::init_idt_and_pics();
    
    // 2.2. 💾 Inicializar Paging e Heap (Usando PMM para alocação de frames)
    // O PMM deve ser preenchido com as informações do Multiboot2 (multiboot2_info_ptr)
    // Inicialização do PMM (vazio, para ser preenchido no init_paging_and_heap)
    let pmm_allocator = memory::frame_alloc::PhysicalMemoryManager::new();

    match unsafe { 
        memory::paging::init_paging_and_heap(
            multiboot2_info_ptr, 
            pmm_allocator, 
            KERNEL_HEAP_START, 
            HEAP_SIZE
        )
    } {
        Ok(_) => {
            println!("[MMU] Paging e Heap inicializados com sucesso.");
            // Exemplo de teste de alocação de Heap (agora é seguro usar o Heap)
            // memory::paging::run_memory_tests(); 
        },
        Err(e) => {
            println!("[FATAL] Falha na inicialização da Memória: {:?}", e);
            loop { unsafe { x86_64::instructions::hlt(); } } // Travar
        }
    }
    
    // 2.3. Inicializar IPC
    ipc::initialize();

    // 3. Inicializar Drivers (Exemplo: Display)
    // Nota: O Framebuffer deve estar mapeado via Paging (Passo 2.2) para ser acessível.
    let display_info = drivers::display::FramebufferInfo {
        address: 0xFFFF_8000_0100_0000 + 0xE000_0000, // Exemplo: FB_PHYS + KERNEL_OFFSET (Endereço Virtual Mapeado)
        width: 1024, height: 768, pitch: 1024 * 4, bpp: 32,
    };
    
    unsafe {
        match drivers::display::DisplayDriver::new(display_info) {
            Ok(mut driver) => {
                match driver.initialize() {
                    Ok(_) => println!("[DRIVER] DisplayDriver pronto e tela limpa."),
                    Err(_) => println!("[ERROR] Falha ao inicializar o DisplayDriver!"),
                }
            }
            Err(_) => println!("[ERROR] Não foi possível criar o DisplayDriver."),
        }
    }
    
    // Loop principal do Kernel: O Kernel nunca deve terminar
    println!("\nLightOS Kernel rodando. Entrando em loop infinito...");
    
    loop {
        // Aguarda a próxima interrupção (temporizador, teclado, etc.)
        unsafe {
            x86_64::instructions::hlt();
        }
    }
}

// ... (Tratamento de Pânico permanece o mesmo) ...
