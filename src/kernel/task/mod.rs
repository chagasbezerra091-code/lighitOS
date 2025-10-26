// src/kernel/interrupts/mod.rs

//! Subsistema de Gerenciamento de Interrupções e Exceções para o LightOS.
//! 
//! Configura a IDT, inicializa o PIC e lida com o despacho de todas as exceções
//! da CPU (0-31) e interrupções de hardware (32+).

use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};
use x86_64::registers::control::Cr2;
use lazy_static::lazy_static; 
use spin::Mutex; 

// Módulos internos
pub mod pic;
use crate::task; // ⬅️ Integração com o Agendador de Tarefas

// ------------------------------------------------------------------------
// --- Definições de Interrupção e Constantes ---
// ------------------------------------------------------------------------

/// ⚡ Vetores de interrupção (IRQs de hardware).
pub const PIC_1_OFFSET: u8 = 32; // IRQs do Master PIC começam em 32
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // IRQs do Slave PIC começam em 40

/// Enumeração de IRQs de hardware (para uso fácil no código Rust)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,           // IRQ0: Temporizador (PIT)
    Keyboard = PIC_1_OFFSET + 1,    // IRQ1: Teclado PS/2
    Serial2 = PIC_1_OFFSET + 3,     // IRQ3: Porta Serial COM2/COM4
    ATA_Primary = PIC_1_OFFSET + 14,// IRQ14: Canal ATA Primário
    Syscall = 0x80,                 // Vetor de Chamada de Sistema (Comum)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

// ------------------------------------------------------------------------
// --- IDT (Interrupt Descriptor Table) ---
// ------------------------------------------------------------------------

lazy_static! {
    /// 🛡️ Tabela de Descritores de Interrupção estática e thread-safe.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // --- Handlers de Exceções da CPU (Vetor 0-31) ---
        // Exceções irrecuperáveis ou críticas devem ser handled primeiro.
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.double_fault.set_handler_fn(double_fault_handler); 
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        
        // Falha de Página (Page Fault) - Essencial para o MMU
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        // --- Handlers de Hardware (Vetor 32+) ---
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        
        // Chamada de Sistema (Syscall) - Exigiria um handler Assembly/FFI dedicado
        // idt[InterruptIndex::Syscall.as_u8()].set_handler_fn(syscall_handler);
        
        idt
    };
}

/// ⚙️ Inicializa o subsistema de interrupções.
pub fn init_idt_and_pics() {
    // 1. Carregar a IDT na CPU
    IDT.load();

    // 2. Configurar os Controladores PIC
    unsafe {
        pic::PICS.lock().initialize();
    }
    
    // 3. Habilitar Interrupções
    x86_64::instructions::interrupts::enable();
}


// ------------------------------------------------------------------------
// --- Handlers de Exceções da CPU (Externa: "x86-interrupt") ---
// ------------------------------------------------------------------------

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    crate::println!("--- KERNEL FATAL: EXCEÇÃO DE DIVISÃO POR ZERO ---");
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) {
    crate::println!("--- KERNEL FATAL: FALHA DE PROTEÇÃO GERAL (GPF) ---");
    crate::println!("Código de Erro: {:#x}", _error_code);
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64
) -> ! {
    // A Dupla Falha (Double Fault) é irrecuperável e exige a reinicialização.
    crate::println!("--- KERNEL FATAL: DUPLA FALHA ---");
    crate::println!("Stack Frame: {:#?}", stack_frame);
    loop {} 
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    // Lógica crucial para o MMU (Page Fault Handler)
    crate::println!("--- EXCEÇÃO: FALHA DE PÁGINA ---");
    crate::println!("Endereço de acesso (CR2): {:?}", Cr2::read());
    crate::println!("Código de Erro: {:?}", error_code);
    crate::println!("Stack Frame: {:#?}", stack_frame);
    
    // Se for uma falha de página não resolúvel (ex: erro de código), trava o kernel
    loop { x86_64::instructions::hlt(); }
}


// ------------------------------------------------------------------------
// --- Handlers de Interrupções de Hardware (IRQs) ---
// ------------------------------------------------------------------------

extern "x86-interrupt" fn timer_interrupt_handler(mut stack_frame: InterruptStackFrame) {
    // Esta é a função que habilita o preempção (preemptive scheduling).
    
    // 1. Chamar o Scheduler para alternar a próxima tarefa
    unsafe {
        // NOTA: Em um kernel real, o salvamento/restauração do TaskContext
        // seria feito em um pequeno trecho de Assembly antes de chamar o Rust.
        // Aqui, simulamos a chamada ao Agendador.
        // task::schedule_next(context_pointer); 
    }
    
    // 2. Enviar EOI (End of Interrupt) para o PIC
    unsafe {
        // Enviar EOI para o Master PIC (pois IRQ0 está no Master)
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // 1. Ler o scancode da porta de dados do PS/2
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // 2. Processar (Ex: Despachar para o Driver de Teclado/Input)
    crate::println!("[INPUT] Scancode: {:#x}", scancode);

    // 3. Enviar EOI
    unsafe {
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
