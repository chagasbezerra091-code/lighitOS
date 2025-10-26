// Exemplo de FFI no Kernel Rust lib.rs

use ipc::{Message, Endpoint, IpcResult, send_message, receive_message};

#[no_mangle]
// Função C: int send_ipc_message(uint64_t dest_id, const Message* msg_ptr)
pub extern "C" fn send_ipc_message(dest_id: u64, msg_ptr: *const Message) -> u32 {
    let destination = Endpoint(dest_id);
    
    // SAFETY: Assume que msg_ptr é um ponteiro não nulo e válido para um Message.
    let msg = unsafe { 
        if msg_ptr.is_null() {
            return ipc::IpcError::InvalidMessage as u32; 
        }
        *msg_ptr
    };

    match send_message(destination, msg) {
        Ok(_) => 0, // Sucesso
        Err(e) => e as u32, // Retorna o código de erro
    }
}

// Função C: int receive_ipc_message(uint64_t receiver_id, Message* out_msg_ptr)
// ... e assim por diante
