// src/kernel/ipc/message.rs

use core::fmt;

/// Tipo de Erro Específico para o Subsistema IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// O endpoint de destino não foi encontrado ou está fechado.
    EndpointNotFound,
    /// O endpoint de destino está em estado inválido para a operação (ex: caixa de entrada cheia).
    InvalidEndpointState,
    /// Falha na desserialização ou validação da mensagem.
    InvalidMessage,
    /// Tempo limite (timeout) na operação de envio/recebimento.
    Timeout,
    /// Erro interno do Kernel.
    InternalError,
}

/// Tipo de Resultado IPC. 
pub type IpcResult<T> = Result<T, IpcError>;

/// Tipo de Mensagem IPC para o LightOS.
/// O tamanho total deve ser mantido pequeno (ex: 64 ou 128 bytes) para performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Garante compatibilidade com C (ABI)
pub struct Message {
    /// O remetente da mensagem.
    pub sender: Endpoint,
    /// O tipo ou propósito da mensagem.
    pub kind: IpcKind,
    /// O payload da mensagem (o dado real).
    /// Usamos um array de bytes para flexibilidade e tamanho fixo.
    pub payload: [u8; 48], 
}

/// Enumeração do Tipo de Mensagem IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // Garante tamanho fixo para C.
pub enum IpcKind {
    Request = 1,
    Response = 2,
    Notification = 3,
    // Tipos de mensagens específicas
    DriverCommand = 10,
    FilesystemRequest = 11,
}

/// Identificador de um Processo, Thread ou Serviço do Kernel.
/// Um 'Endpoint' é o endereço de comunicação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Representa o tipo como apenas um u64, útil para FFI.
pub struct Endpoint(pub u64);

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IPC(De: {}, Tipo: {:?}, Tamanho: {} bytes)",
            self.sender.0,
            self.kind,
            self.payload.len()
        )
    }
}
