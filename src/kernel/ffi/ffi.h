// src/kernel/ffi/ffi.h

#ifndef LIGHTOS_FFI_H
#define LIGHTOS_FFI_H

#include <stdint.h>
#include <stddef.h>

// ------------------------------------------------------------------------
// --- Tipos de Dados Compartilhados (De src/kernel/ipc/message.rs) ---
// ------------------------------------------------------------------------

// A 'Endpoint' é representada como um u64 (uint64_t)
typedef uint64_t Endpoint;

// Enumeração do Tipo de Mensagem IPC (Deve corresponder ao #[repr(u32)] em Rust)
typedef enum {
    IPC_KIND_REQUEST           = 1,
    IPC_KIND_RESPONSE          = 2,
    IPC_KIND_NOTIFICATION      = 3,
    IPC_KIND_DRIVER_COMMAND    = 10,
    IPC_KIND_FILESYSTEM_REQUEST= 11,
} IpcKind;

// Estrutura de Mensagem IPC (Deve corresponder ao #[repr(C)] em Rust)
// Certifique-se de que os campos e o alinhamento correspondam à struct Message em Rust.
#define IPC_PAYLOAD_SIZE 48
typedef struct {
    Endpoint sender;
    IpcKind kind;
    uint8_t payload[IPC_PAYLOAD_SIZE];
} Message;

// Códigos de Erro IPC (De src/kernel/ipc/message.rs)
typedef enum {
    IPC_ERROR_SUCCESS           = 0, // Convenção C: 0 é sucesso
    IPC_ERROR_ENDPOINT_NOT_FOUND = 1,
    IPC_ERROR_INITIALIZATION_FAILED = 2, // Generalizado para outros erros
    IPC_ERROR_INVALID_MESSAGE    = 3,
    IPC_ERROR_TIMEOUT            = 4,
    IPC_ERROR_INTERNAL_ERROR     = 5,
    // Note que os códigos de erro Touchscreen/Drivers também usam uint32_t e podem
    // ser mapeados a partir de um conjunto de valores base.
    DRIVER_ERROR_BASE           = 100,
    DRIVER_ERROR_DEVICE_NOT_FOUND = DRIVER_ERROR_BASE + 1,
} LightOSErrorCode;

// ------------------------------------------------------------------------
// --- Funções Rust Expostas ao C (lightos_ffi/mod.rs) ---
// ------------------------------------------------------------------------

/**
 * @brief Envia uma mensagem IPC para um endpoint de destino.
 * * @param dest_id O ID do endpoint de destino.
 * @param msg_ptr Ponteiro para a struct Message a ser enviada.
 * @return LightOSErrorCode (0 em caso de sucesso).
 */
uint32_t lightos_ipc_send(uint64_t dest_id, const Message* msg_ptr);

/**
 * @brief Recebe uma mensagem IPC para o endpoint receptor.
 * * @param receiver_id O ID do endpoint receptor.
 * @param out_msg_ptr Ponteiro de saída onde a mensagem recebida será escrita.
 * @return LightOSErrorCode (0 em caso de sucesso).
 */
uint32_t lightos_ipc_receive(uint64_t receiver_id, Message* out_msg_ptr);

/**
 * @brief Inicializa e verifica o driver de Touchscreen.
 * * @param mmio_addr Endereço base de MMIO do dispositivo.
 * @return LightOSErrorCode (0 em caso de sucesso).
 */
uint32_t lightos_driver_touch_init(uintptr_t mmio_addr);


// ------------------------------------------------------------------------
// --- Funções C (Stubs) Chamadas pelo Rust (Para serem implementadas em C) ---
// ------------------------------------------------------------------------

/**
 * @brief Função de log de baixo nível implementada em C.
 * Usada por subsistemas Rust para imprimir mensagens.
 * * @param severity Nível de severidade do log (ex: 1=INFO, 2=WARN, 3=ERROR).
 * @param message_ptr Ponteiro para a string (bytes) da mensagem.
 * @param len Comprimento da mensagem.
 */
void lightos_c_log(uint32_t severity, const uint8_t* message_ptr, size_t len);

/**
 * @brief Função C para ler um byte de uma porta I/O (inb).
 * @param port A porta I/O (0-0xFFFF).
 * @return O byte lido.
 */
uint8_t lightos_io_inb(uint16_t port);

/**
 * @brief Função C para escrever um byte em uma porta I/O (outb).
 * @param port A porta I/O (0-0xFFFF).
 * @param data O byte a ser escrito.
 */
void lightos_io_outb(uint16_t port, uint8_t data);

/**
 * @brief Função C para configurar o Paging/MMU inicial.
 * @return 0 em caso de sucesso, != 0 em caso de erro.
 */
uint32_t lightos_mmu_setup_paging();

#endif // LIGHTOS_FFI_H
