// src/kernel/ipc/mod.rs

//! Subsistema de Interprocess Communication (IPC) do LightOS.
//! 
//! Fornece mecanismos de troca de mensagens e sincronização entre processos.

// Importa os submódulos
mod message;
mod manager;

// Exporta tipos e funções públicas
pub use message::{Message, IpcError, IpcResult, Endpoint, IpcKind};
pub use manager::{IpcManager, send_message, receive_message, register_endpoint};

// Funções de inicialização do subsistema IPC
// Esta é a função que o código de inicialização do Kernel C/Rust chamaria.
pub fn initialize() {
    // Inicializa o gerenciador de IPC (se necessário, como tabelas de lookup)
    IpcManager::init();
    crate::println!("INFO: Subsistema IPC inicializado.");
}
