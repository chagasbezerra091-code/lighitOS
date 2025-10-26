// src/bootloader/bootloader.c
#include <stdint.h>
#include <stddef.h>

// ------------------------------------------------------------------------
// --- Funções C/Rust Externas ---
// ------------------------------------------------------------------------

// A função de entrada principal do Kernel Rust (o coração do seu SO).
// Assinatura: extern "C" fn kernel_main(multiboot2_info_ptr: u64);
extern void kernel_main(uint64_t);

// Função simples de print para debug.
extern void lightos_early_print(const char* str);

// ------------------------------------------------------------------------
// --- Definições de Configuração (Endereços Físicos) ---
// ------------------------------------------------------------------------

// Endereço onde o Kernel (e os dados do Multiboot) serão mapeados temporariamente.
#define KERNEL_MAPPING_ADDR 0x0000000000100000 // 1MB

// ------------------------------------------------------------------------
// --- Função de Entrada C em 64 bits ---
// ------------------------------------------------------------------------

/**
 * @brief Função de entrada do Bootloader C.
 * * Esta função é chamada a partir do código Assembly após a transição para 
 * o Modo Longo (64 bits).
 *
 * @param multiboot2_info_ptr Endereço físico da estrutura de informações do Multiboot2.
 */
void c_boot_entry(uint64_t multiboot2_info_ptr) {
    
    // O ponteiro do Multiboot2 é um endereço físico. 
    // Em um Kernel real, este ponteiro seria mapeado para um endereço virtual
    // antes de ser acessado.

    // 1. Inicialização do Early Log (Função C ou Assembly)
    lightos_early_print("LightOS Bootloader: Entrando em c_boot_entry (Modo Longo).\n");
    
    // 2. Inicialização do MMU/Paging (A ser implementada em C ou Assembly)
    // A função lightos_mmu_setup_paging é declarada no ffi.h
    // if (lightos_mmu_setup_paging() != 0) {
    //     lightos_early_print("ERRO: Falha na inicializacao do Paging.\n");
    //     while(1){}
    // }
    
    lightos_early_print("LightOS Bootloader: Paging/MMU inicializado.\n");

    // 3. Chamada final para o Kernel Rust
    lightos_early_print("LightOS Bootloader: Transferindo controle para kernel_main (Rust).\n");
    
    // Passamos o ponteiro de info do Multiboot2 diretamente para o Kernel Rust
    kernel_main(multiboot2_info_ptr);

    // Se o kernel_main retornar (o que não deve ocorrer em um Kernel), entramos em loop.
    lightos_early_print("ERRO: kernel_main retornou.\n");
    while (1) {}
}

// ------------------------------------------------------------------------
// --- Funções Auxiliares (Stubs para o FFI) ---
// ------------------------------------------------------------------------

// Stubs simples para satisfazer o lightos_ffi/mod.rs e outros.
// Você precisará implementá-los no seu código de baixo nível C/Assembly real.

void lightos_c_log(uint32_t severity, const uint8_t* message_ptr, size_t len) {
    // Implementação de log de baixo nível: escreve diretamente na serial ou VGA
    // Por enquanto, apenas um placeholder para satisfazer o link.
}

// ... stubs para lightos_io_inb, lightos_io_outb, lightos_mmu_setup_paging, etc.
// ... (omissão para brevidade, mas devem ser implementados no seu código C/Assembly)
