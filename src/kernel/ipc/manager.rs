// src/kernel/ipc/manager.rs

use super::message::{Message, IpcResult, Endpoint};
use spin::{Mutex, Once}; 

/// 📚 Tabela Global para mapear Endpoints para Endereços de Caixa de Entrada.
/// Usamos 'Once' e 'Mutex' para garantir uma inicialização segura e acesso thread-safe.
/// O Kernel real usaria estruturas mais complexas e eficientes.
static ENDPOINT_MAP: Once<Mutex<SimpleEndpointMap>> = Once::new();

/// Estrutura Simples de Mapeamento (Apenas para fins de demonstração)
/// Em um Kernel real, esta seria uma Hash Map ou Árvore B de alta performance.
pub struct SimpleEndpointMap {
    // Array simples para endpoints estáticos
    map: [(Endpoint, Option<Message>); 16],
    count: usize,
}

impl SimpleEndpointMap {
    const fn new() -> Self {
        // Inicializa o mapa com endpoints vazios
        const EMPTY: (Endpoint, Option<Message>) = (Endpoint(0), None);
        SimpleEndpointMap { map: [EMPTY; 16], count: 0 }
    }

    /// Tenta registrar um novo endpoint.
    fn register(&mut self, endpoint: Endpoint) -> IpcResult<()> {
        if self.count >= self.map.len() {
            return Err(IpcError::InternalError); // Tabela cheia
        }
        
        // Verifica se o endpoint já existe
        if self.map[0..self.count].iter().any(|(e, _)| *e == endpoint) {
            return Err(IpcError::InternalError); // Já registrado
        }

        self.map[self.count] = (endpoint, None);
        self.count += 1;
        Ok(())
    }

    /// Tenta enviar uma mensagem (simplesmente armazena na "caixa de entrada").
    fn send(&mut self, destination: Endpoint, msg: Message) -> IpcResult<()> {
        if let Some(entry) = self.map[0..self.count].iter_mut().find(|(e, _)| *e == destination) {
            if entry.1.is_none() {
                entry.1 = Some(msg);
                Ok(())
            } else {
                Err(IpcError::InvalidEndpointState) // Caixa de entrada cheia (simples)
            }
        } else {
            Err(IpcError::EndpointNotFound)
        }
    }

    /// Tenta receber uma mensagem (retira da "caixa de entrada").
    fn receive(&mut self, receiver: Endpoint) -> IpcResult<Message> {
        if let Some(entry) = self.map[0..self.count].iter_mut().find(|(e, _)| *e == receiver) {
            if let Some(msg) = entry.1.take() {
                Ok(msg)
            } else {
                Err(IpcError::Timeout) // Nenhuma mensagem (timeout/polling simples)
            }
        } else {
            Err(IpcError::EndpointNotFound)
        }
    }
}

/// 🧠 Gerenciador de IPC (Singleton)
pub struct IpcManager;

impl IpcManager {
    /// Inicializa a tabela de mapeamento global.
    pub fn init() {
        ENDPOINT_MAP.call_once(|| Mutex::new(SimpleEndpointMap::new()));
        // Exemplo: Registra o Kernel Log Endpoint (ID 1)
        let _ = register_endpoint(Endpoint(1)); 
    }
}

// ------------------------------------------------------------------------
// --- API Pública do IPC (Usada por C e Rust) ---
// ------------------------------------------------------------------------

/// 📬 Envia uma mensagem para o `destination` endpoint.
///
/// Este seria o ponto de sincronização onde a thread chamadora poderia ser
/// bloqueada até o envio ser concluído.
pub fn send_message(destination: Endpoint, msg: Message) -> IpcResult<()> {
    if let Some(map) = ENDPOINT_MAP.get() {
        let mut map_lock = map.lock();
        map_lock.send(destination, msg)
    } else {
        Err(IpcError::InternalError) // Não inicializado
    }
}

/// 📥 Tenta receber uma mensagem para o `receiver` endpoint.
///
/// Este seria o ponto de sincronização onde a thread receptora poderia ser
/// bloqueada esperando por uma mensagem.
pub fn receive_message(receiver: Endpoint) -> IpcResult<Message> {
    if let Some(map) = ENDPOINT_MAP.get() {
        let mut map_lock = map.lock();
        map_lock.receive(receiver)
    } else {
        Err(IpcError::InternalError) // Não inicializado
    }
}

/// 📝 Registra um novo endpoint no sistema.
pub fn register_endpoint(endpoint: Endpoint) -> IpcResult<()> {
    if let Some(map) = ENDPOINT_MAP.get() {
        let mut map_lock = map.lock();
        map_lock.register(endpoint)
    } else {
        Err(IpcError::InternalError)
    }
}
