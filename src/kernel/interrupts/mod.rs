// src/kernel/interrupts/mod.rs

//! Subsistema de Gerenciamento de Interrupções e Exceções para o LightOS.

use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};
// ... (Outros imports permanecem os mesmos) ...
use x86_64::registers::control::Cr2;
use lazy_static::lazy_static; 
use spin::Mutex; 

pub mod pic;
use crate::task; 
use crate::syscall; // Novo import para o dispatcher de Syscalls

// ... (Constantes e Enumerações InterruptIndex permanecem as mesmas) ...

// ------------------------------------------------------------------------
// --- Funções Assembly (Declaradas como Extern) ---
// ------------------------------------------------------------------------

extern "C" {
    /// O ponto de entrada Assembly para a interrupção do Timer (IRQ0).
    /// Ele salva o contexto, chama o handler Rust (`lightos_timer_handler_rust`),
    /// e restaura/troca o contexto.
    pub fn lightos_timer_handler();

    /// O ponto de entrada Assembly para a interrupção do Teclado (IRQ1).
    pub fn lightos_keyboard_handler();

    /// O ponto de entrada Assembly para Chamadas de Sistema (INT 0x80 ou SYSCALL).
    pub fn lightos_syscall_entry(); 
}

// ------------------------------------------------------------------------
// --- IDT (Interrupt Descriptor Table) - Configuração ---
// ------------------------------------------------------------------------

lazy_static! {
    /// 🛡️ Tabela de Descritores de Interrupção estática e thread-safe.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // --- Handlers de Exceções da CPU (Vetor 0-31) ---
        // Exceções irrecuperáveis ou críticas (permanecem como funções Rust)
        // ... (divide_error, double_fault, general_protection_fault, page_fault, etc.)
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.double_fault.set_handler_fn(double_fault_handler); 
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        // --- Handlers de Hardware (Vetor 32+) - Apontam para o Assembly Wrapper ---
        
        // 1. Timer IRQ (IRQ0 = 32)
        // O handler Assembly fará a troca de contexto.
        unsafe {
            use x86_64::structures::idt::HandlerFunc;
            idt[InterruptIndex::Timer.as_u8()].set_handler_fn(
                core::mem::transmute(lightos_timer_handler as *const ())
            );
        }
        
        // 2. Keyboard IRQ (IRQ1 = 33)
        unsafe {
            use x86_64::structures::idt::HandlerFunc;
            idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(
                core::mem::transmute(lightos_keyboard_handler as *const ())
            );
        }
        
        // 3. Syscall (INT 0x80 = 0x80)
        // Embora `SYSCALL` seja moderno, a `INT 0x80` é uma alternativa comum para a IDT.
        unsafe {
            use x86_64::structures::idt::HandlerFunc;
            idt[InterruptIndex::Syscall.as_u8()].set_handler_fn(
                core::mem::transmute(lightos_syscall_entry as *const ())
            ).set_present(true)
             .disable_interrupts(false)
             .set_privilege_level(x86_64::PrivilegeLevel::Ring3); // Permite chamada do Userspace
        }
        
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
    crate::println!("INFO: Interrupções (IDT/PIC) inicializadas.");
}


// ------------------------------------------------------------------------
// --- Handlers de Exceções da CPU (Permanecem os mesmos) ---
// ------------------------------------------------------------------------
// ... (divide_error_handler, double_fault_handler, general_protection_fault_handler, page_fault_handler)
// ... (Seu código original deve ser mantido aqui)
// ...

// ------------------------------------------------------------------------
// --- Handlers de IRQ (Chamados pelo Assembly Wrapper - extern "C") ---
// ------------------------------------------------------------------------

/// ⏰ Handler Rust para a interrupção do Timer (IRQ0).
/// * Chamado pelo Assembly wrapper `lightos_timer_handler`.
#[no_mangle]
pub extern "C" fn lightos_timer_handler_rust(mut context_ptr: *mut task::TaskContext) {
    
    // 1. Logar (Opcional)
    // crate::println!(".");

    // 2. Chamar o Scheduler para realizar a troca
    let current_context = unsafe { &mut *context_ptr };
    
    // O Scheduler fará o agendamento e modificará `current_context` para o próximo.
    unsafe {
        task::schedule_next(current_context);
    }

    // 3. Enviar EOI
    unsafe {
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    
    // O Assembly Wrapper continua daqui, restaurando o contexto da próxima tarefa.
}

/// ⌨️ Handler Rust para a interrupção do Teclado (IRQ1).
#[no_mangle]
pub extern "C" fn lightos_keyboard_handler_rust() {
    use x86_64::instructions::port::Port;

    // 1. Ler o scancode
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    // 2. Processar (Ex: Despachar para o Driver de Teclado/Input)
    crate::println!("[INPUT] Scancode: {:#x}", scancode);

    // 3. Enviar EOI
    unsafe {
        pic::PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}


// ------------------------------------------------------------------------
// --- PIC: Peripheral Interrupt Controller (Manter o módulo pic.rs original) ---
// ------------------------------------------------------------------------
