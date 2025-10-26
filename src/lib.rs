// src/lib.rs
#![no_std] // Não usa a biblioteca padrão Rust (essencial para kernels)
#![no_main] // Não usa a interface principal padrão do Rust (para definir nosso próprio ponto de entrada)
#![feature(custom_test_frameworks)] // Usado para testes unitários no futuro
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)] // Permite código não usado para fins de demonstração

use core::panic::PanicInfo;

// ------------------------------------------------------------------------
// --- Módulos do Kernel ---
// ------------------------------------------------------------------------

pub mod RustKernelConfig; // Configurações HAL
pub mod ipc;            // Comunicação Interprocessos
pub mod drivers;        // Drivers de Hardware
pub mod ffi;            // Interface de Função Estrangeira (C/Rust Bridge)

// Reexporta as configurações HAL específicas da arquitetura
#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64/x86_64_arch.hal"]
pub mod arch_hal;

// ------------------------------------------------------------------------
// --- FUNÇÕES DE SAÍDA (Logging) ---
// ------------------------------------------------------------------------

/// Macro simples para imprimir logs usando o FFI do C (console.cc).
/// A macro usa a função FFI lightos_c_log que é implementada em C++.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::log_to_c(format_args!($($arg)*)));
}

/// Helper para enviar o FormatArgs do Rust para o console C/C++.
#[doc(hidden)]
pub fn log_to_c(args: core::fmt::Arguments) {
    use core::fmt::Write;
    // Buffer temporário para formatar a string antes de enviar (evita alocação de heap)
    let mut buffer = [0u8; 256]; 
    let mut writer = FfiLogWriter { 
        buffer: &mut buffer, 
        len: 0 
    };
    
    // Formata a string no buffer
    if writer.write_fmt(args).is_ok() {
        // SAFETY: Chamada segura para o FFI C (console.cc)
        // O nível de severidade 1 é LOG_LEVEL_INFO
        unsafe {
            ffi::lightos_c_log(1, writer.buffer.as_ptr(), writer.len);
        }
    }
}

/// Writer que acumula bytes e chama o FFI C.
struct FfiLogWriter<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl<'a> core::fmt::Write for FfiLogWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remaining = self.buffer.len() - self.len;

        if bytes.len() > remaining {
            // Buffer cheio: Truncar a mensagem (ou enviar a parte preenchida e resetar)
            return Err(core::fmt::Error); 
        }

        // Copia os bytes para o buffer
        self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

// ------------------------------------------------------------------------
// --- PONTO DE ENTRADA DO KERNEL (Chamado pelo bootloader.c) ---
// ------------------------------------------------------------------------

/// 🏁 O Ponto de Entrada principal do Kernel LightOS (Rust).
/// 
/// Esta função é chamada a partir do código C (c_boot_entry) em Modo Longo (64-bit).
///
/// # Argumentos
/// * `multiboot2_info_ptr`: Endereço físico da estrutura de informações do Multiboot2.
#[no_mangle]
pub extern "C" fn kernel_main(multiboot2_info_ptr: u64) -> ! {
    
    // SAFETY: Precisamos do endereço VGA para inicializar o console C++.
    // Assumimos que o endereço VGA está mapeado e é acessível.
    let vga_addr = RustKernelConfig::VGA_TEXT_BUFFER_ADDR as usize;
    unsafe {
        // Chama a função C++ para inicializar o console VGA
        // lightos_driver_console_init(uintptr_t vga_addr);
        // Usamos um stub FFI aqui, que seria implementado no console.cc ou .h
        // A função Console::initialize(uintptr_t) deve ser exposta ou chamada via stub.
    }
    
    // Simplesmente para logar que estamos no Rust:
    println!("----------------------------------------------------------");
    println!("LightOS: Controle transferido para kernel_main (Rust).");
    println!("Multiboot2 Info Ptr: {:#x}", multiboot2_info_ptr);
    println!("Arquitetura: {}", arch_hal::ARCH_NAME);
    println!("----------------------------------------------------------");

    // 1. Inicializar Subsistemas:
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
    
    // Loop principal do Kernel: O Kernel nunca deve terminar
    println!("\nLightOS Kernel rodando. Entrando em loop infinito...");
    
    loop {
        // Aqui o Kernel ficaria ocioso (halt) ou processaria interrupções.
        // O x86_64 hlt (halt) é necessário para economizar energia.
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

// ------------------------------------------------------------------------
// --- Tratamento de Pânico (Obrigatório em no_std) ---
// ------------------------------------------------------------------------

/// Função chamada em caso de pânico (erro grave e irrecuperável).
/// * Esta função é crítica e não deve retornar.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Nível de severidade 3 é LOG_LEVEL_ERROR
    
    // SAFETY: Chamada direta ao C/C++ FFI para logar o pânico.
    unsafe {
        ffi::lightos_c_log(3, "--- KERNEL PANIC ---".as_ptr(), 20);
        if let Some(location) = info.location() {
            let msg = format_args!("Local: {}:{}\n", location.file(), location.line());
            log_to_c(msg);
        }
        if let Some(message) = info.message() {
            let msg = format_args!("Mensagem: {:?}\n", message);
            log_to_c(msg);
        } else {
            ffi::lightos_c_log(3, "Mensagem de pânico indisponível.\n".as_ptr(), 30);
        }
        ffi::lightos_c_log(3, "----------------------".as_ptr(), 20);
    }
    
    // Deve entrar em loop infinito após o pânico
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
