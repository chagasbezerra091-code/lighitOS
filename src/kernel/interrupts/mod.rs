// src/kernel/interrupts/mod.rs

//! Subsistema de Gerenciamento de Interrupções e Exceções para o LightOS.
//! 
//! Responsável por configurar a IDT (Interrupt Descriptor Table) e lidar
//! com interrupções de hardware e exceções da CPU.

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static; // Para inicialização estática e thread-safe da IDT
use spin::Mutex; // Para acesso seguro ao PIC

// Importa o driver PIC
pub mod pic;

// ------------------------------------------------------------------------
// --- Definições de Interrupção ---
// ------------------------------------------------------------------------

/// ⚡ Vetores de interrupção (IRQs de hardware).
/// Os vetores 0-31 são reservados para exceções da CPU.
/// Usamos vetores a partir de 32 para evitar conflitos com as exceções.
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

/// Enumeração de IRQs de hardware (para uso fácil no código Rust)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,           // IRQ0: Timer
    Keyboard = PIC_1_OFFSET + 1,    // IRQ1: Teclado PS/2
}

// ------------------------------------------------------------------------
// --- IDT (Interrupt Descriptor Table) ---
// ------------------------------------------------------------------------

lazy_static! {
    /// Tabela de Descritores de Interrupção (IDT) estática.
    /// É inicializada apenas uma vez e é acessada de forma thread-safe.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // --- Handlers de Exceções da CPU (Vetor 0-31) ---
        // 0. Falha de Divisão (Division By Zero)
        idt.divide_error.set_handler_fn(divide_error_handler);
        // 14. Falha de Página (Page Fault) - Crucial para o MMU
        idt.page_fault.set_handler_fn(page_fault_handler);
        // 8. Dupla Falha (Double Fault) - Se uma exceção ocorrer durante o tratamento de outra
        idt.double_fault.set_handler_fn(double_fault_handler);
        // 13. Exceção de Proteção Geral (General Protection Fault - GPF)
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        
        // --- Handlers de Hardware (Vetor 32+) ---
        // IRQ 0: Temporizador
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        // IRQ 1: Teclado
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        
        // Adicionar a interrupção de Chamada de Sistema (Syscall) - Vetor 0x80 (128)
        // idt[0x80].set_handler_fn(syscall_handler); // Exigiria um handler assembly
        
        idt
    };
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

/// ⚙️ Inicializa o subsistema de interrupções.
/// * Carrega a IDT na CPU e configura o PIC.
pub fn init_idt_and_pics() {
    // 1. Carregar a IDT na CPU
    IDT.load();
    crate::println!("INFO: IDT carregada com sucesso.");

    // 2. Configurar os Controladores PIC
    unsafe {
        pic::PICS.lock().initialize();
    }
    crate::println!("INFO: PICs remapeados e inicializados.");
    
    // 3. Habilitar Interrupções
    x86_64::instructions::interrupts::enable();
    crate::println!("INFO: Interrupções habilitadas (CLI).");
}

// ------------------------------------------------------------------------
// --- Handlers de Exceções da CPU ---
// ------------------------------------------------------------------------

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEÇÃO: DIVISÃO POR ZERO");
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    crate::println!("EXCEÇÃO: FALHA DE PÁGINA");
    crate::println!("Endereço de acesso: {:?}", Cr2::read());
    crate::println!("Código de Erro: {:?}", error_code);
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    // A Dupla Falha (Double Fault) é irrecuperável
    crate::println!("EXCEÇÃO: DUPLA FALHA");
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop {} // Loop infinito sem hlt para evitar loop de falhas
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) {
    crate::println!("EXCEÇÃO: FALHA DE PROTEÇÃO GERAL (GPF)");
    crate::println!("Código de Erro: {}", _error_code);
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}


// ------------------------------------------------------------------------
// --- Handlers de Interrupções de Hardware (IRQs) ---
// ------------------------------------------------------------------------

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // A cada tick do temporizador, o agendador de tarefas será chamado.
    // crate::println!("."); // Descomentar para ver os ticks

    // 1. Enviar EOI (End of Interrupt) para o PIC Mestre
    unsafe {
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // 1. Ler o scancode da porta de dados do PS/2 (0x60)
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // 2. Processar o scancode
    crate::println!("Teclado IRQ! Scancode: {:#x}", scancode);

    // 3. Enviar EOI
    unsafe {
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
