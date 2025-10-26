// src/kernel/console.h
#ifndef LIGHTOS_CONSOLE_H
#define LIGHTOS_CONSOLE_H

#include <stdint.h>
#include <stddef.h>

// ------------------------------------------------------------------------
// --- Definições de Severidade de Log (Para uso em C/C++ e FFI com Rust) ---
// ------------------------------------------------------------------------

/**
 * @brief Níveis de severidade de log (deve corresponder ao enum no Rust, se aplicável).
 */
typedef enum {
    LOG_LEVEL_INFO = 1,
    LOG_LEVEL_WARN = 2,
    LOG_LEVEL_ERROR = 3,
    LOG_LEVEL_DEBUG = 4
} LogLevel;

// ------------------------------------------------------------------------
// --- Classe Console (C++) ---
// ------------------------------------------------------------------------

/**
 * @brief Gerencia a saída de texto para o console do sistema (VGA, Serial, etc.).
 * * Esta classe deve ser um singleton.
 */
class Console {
public:
    /**
     * @brief Obtém a única instância do Console (Singleton).
     */
    static Console& get_instance();

    /**
     * @brief Inicializa o console (configura VGA/Serial).
     * @param vga_addr Endereço base do buffer de vídeo.
     */
    void initialize(uintptr_t vga_addr);

    /**
     * @brief Escreve uma string nula-terminada no console.
     * @param str Ponteiro para a string a ser escrita.
     */
    void print(const char* str);

    /**
     * @brief Escreve um log formatado com nível de severidade.
     * @param level Nível de severidade do log.
     * @param file Nome do arquivo de origem (para debug).
     * @param line Número da linha de origem (para debug).
     * @param message A mensagem principal.
     */
    void log(LogLevel level, const char* file, int line, const char* message);

private:
    Console(); // Construtor privado para Singleton
    Console(const Console&) = delete; // Desabilitar cópia
    Console& operator=(const Console&) = delete; // Desabilitar atribuição

    uint8_t* vga_buffer; // Ponteiro para o buffer de vídeo.
    size_t cursor_x;     // Posição atual do cursor (coluna).
    size_t cursor_y;     // Posição atual do cursor (linha).

    /**
     * @brief Move o cursor para a próxima linha e lida com a rolagem.
     */
    void advance_line();
};

// ------------------------------------------------------------------------
// --- Macros C++ de Conveniência (para uso fácil) ---
// ------------------------------------------------------------------------

#define LOG_INFO(msg)    Console::get_instance().log(LOG_LEVEL_INFO, __FILE__, __LINE__, msg)
#define LOG_WARN(msg)    Console::get_instance().log(LOG_LEVEL_WARN, __FILE__, __LINE__, msg)
#define LOG_ERROR(msg)   Console::get_instance().log(LOG_LEVEL_ERROR, __FILE__, __LINE__, msg)
#define LOG_DEBUG(msg)   Console::get_instance().log(LOG_LEVEL_DEBUG, __FILE__, __LINE__, msg)

// ------------------------------------------------------------------------
// --- FFI para Rust (Implementa a função exigida pelo ffi/mod.rs) ---
// ------------------------------------------------------------------------

/**
 * @brief Implementação em C do stub de log chamado pelo código Rust FFI.
 * * Corresponde a 'lightos_c_log' no ffi/mod.rs
 */
#ifdef __cplusplus
extern "C" {
#endif

void lightos_c_log(uint32_t severity, const uint8_t* message_ptr, size_t len);

#ifdef __cplusplus
}
#endif

#endif // LIGHTOS_CONSOLE_H
