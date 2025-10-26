// src/kernel/ffi/mod.rs

//! Módulo de Foreign Function Interface (FFI) do LightOS.
//! 
//! Gerencia a interoperabilidade entre as partes C e Rust do Kernel.

use crate::{
    ipc::{self, Endpoint, Message, IpcError}, 
    drivers::{
        touchscreen::{self, TouchscreenDriver, TouchscreenError},
    },
    RustKernelConfig,
};

// ------------------------------------------------------------------------
// --- Funções Rust Expostas ao Código C ---
// ------------------------------------------------------------------------

/// 📬 Wrapper de FFI para o envio de mensagens IPC.
/// 
/// Assinatura C: u32 lightos_ipc_send(u64 dest_id, const Message* msg_ptr);
#[no_mangle]
pub extern "C" fn lightos_ipc_send(dest_id: u64, msg_ptr: *const Message) -> u32 {
    let destination = Endpoint(dest_id);
    
    // # SAFETY: Assumimos que 'msg_ptr' é um ponteiro de C válido e não nulo.
    let msg = unsafe { 
        if msg_ptr.is_null() {
            return ipc::IpcError::InvalidMessage as u32; 
        }
        *msg_ptr
    };

    match ipc::send_message(destination, msg) {
        Ok(_) => 0, // Sucesso (0 é a convenção Unix para sucesso)
        Err(e) => e as u32, // Retorna o código de erro IPC
    }
}

/// 📥 Wrapper de FFI para o recebimento de mensagens IPC.
/// 
/// Assinatura C: u32 lightos_ipc_receive(u64 receiver_id, Message* out_msg_ptr);
#[no_mangle]
pub extern "C" fn lightos_ipc_receive(receiver_id: u64, out_msg_ptr: *mut Message) -> u32 {
    let receiver = Endpoint(receiver_id);
    
    // # SAFETY: Assumimos que 'out_msg_ptr' é um ponteiro de C válido e mutável.
    if out_msg_ptr.is_null() {
        return IpcError::InvalidMessage as u32;
    }

    match ipc::receive_message(receiver) {
        Ok(msg) => {
            // Escreve a mensagem recebida de volta na memória fornecida pelo C
            unsafe { core::ptr::write_volatile(out_msg_ptr, msg); }
            0
        }
        Err(e) => e as u32, // Retorna o código de erro IPC (ex: Timeout)
    }
}

/// 👆 Wrapper de FFI para inicializar o driver de Touchscreen.
/// 
/// Assinatura C: u32 lightos_driver_touch_init(uintptr_t mmio_addr);
#[no_mangle]
pub extern "C" fn lightos_driver_touch_init(mmio_addr: usize) -> u32 {
    // Nota: Em um kernel real, o estado do driver seria armazenado em uma estrutura
    // global gerenciada pelo Rust, e não apenas inicializado.
    
    // # SAFETY: Assumimos que 'mmio_addr' é um endereço de hardware válido e mapeado.
    let mut driver = unsafe { 
        match TouchscreenDriver::new(mmio_addr) {
            Ok(d) => d,
            Err(e) => return e as u32,
        }
    };
    
    match driver.init() {
        Ok(_) => 0,
        Err(e) => e as u32,
    }
}


// ------------------------------------------------------------------------
// --- Funções C Acessadas pelo Código Rust ---
// ------------------------------------------------------------------------

/// Bloco 'extern C' para declarar funções C que o Rust precisa chamar.
/// Essas funções C implementam a parte de I/O de baixo nível ou inicialização.
#[allow(improper_ctypes)] // Ignora avisos sobre tipos C não primitivos (se aplicável)
extern "C" {
    // Declaração do logger C (para usar no início do Kernel)
    pub fn lightos_c_log(severity: u32, message_ptr: *const u8, len: usize);
    
    // Funções de I/O de porta (inb/outb)
    pub fn lightos_io_inb(port: u16) -> u8;
    pub fn lightos_io_outb(port: u16, data: u8);

    // Função C para a inicialização da tabela de páginas
    pub fn lightos_mmu_setup_paging() -> u32;
}

// ------------------------------------------------------------------------
// --- Métodos de Conveniência Rust (Usando as funções C acima) ---
// ------------------------------------------------------------------------

/// ✏️ Função de log simples que chama a função C subjacente.
pub fn log_c(severity: u32, message: &str) {
    // # SAFETY: lightos_c_log é uma função FFI que aceita um ponteiro e um comprimento.
    // Garantimos que o ponteiro é válido para 'len' bytes UTF-8 válidos.
    unsafe {
        lightos_c_log(severity, message.as_ptr(), message.len());
    }
}

/// 📤 Wrapper Rust para o I/O de porta `outb`.
pub fn io_outb(port: u16, data: u8) {
    // # SAFETY: Assumimos que lightos_io_outb é uma função C segura que lida com portas I/O.
    unsafe {
        lightos_io_outb(port, data);
    }
}
